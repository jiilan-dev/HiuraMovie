use crate::infrastructure::storage::s3::StorageService;
use anyhow::{anyhow, Result};
use axum::{
    body::Bytes,
    extract::{multipart::Field, Multipart},
};
use futures_util::StreamExt;
use std::io::Cursor;
use tracing::{error, info};

// Minimum part size for S3 is 5MB. We use 6MB to be safe.
const MIN_PART_SIZE: usize = 6 * 1024 * 1024;

pub struct MultipartUploader<'a> {
    storage: &'a StorageService,
    key: String,
    upload_id: String,
    parts: Vec<aws_sdk_s3::types::CompletedPart>,
    part_number: i32,
    buffer: Vec<u8>,
}

impl<'a> MultipartUploader<'a> {
    pub async fn new(storage: &'a StorageService, key: String, content_type: &str) -> Result<Self> {
        let upload_id = storage
            .create_multipart_upload(&key, content_type)
            .await
            .map_err(|e| anyhow!("Failed to initiate upload: {}", e))?;

        Ok(Self {
            storage,
            key,
            upload_id,
            parts: Vec::new(),
            part_number: 1,
            buffer: Vec::with_capacity(MIN_PART_SIZE),
        })
    }

    pub async fn write_chunk(&mut self, chunk: Bytes) -> Result<()> {
        self.buffer.extend_from_slice(&chunk);

        if self.buffer.len() >= MIN_PART_SIZE {
            self.flush_part().await?;
        }

        Ok(())
    }

    async fn flush_part(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let body = Bytes::from(self.buffer.clone()); // Bytes::from is cheap copy (ref count)
        // Reset buffer capacity but clear content
        self.buffer.clear(); 
        // Ensure ability to grow back
        self.buffer.reserve(MIN_PART_SIZE);

        let part = self
            .storage
            .upload_part(&self.key, &self.upload_id, self.part_number, body)
            .await
            .map_err(|e| anyhow!("Failed to upload part {}: {}", self.part_number, e))?;

        self.parts.push(part);
        self.part_number += 1;

        Ok(())
    }

    pub async fn finish(mut self) -> Result<String> {
        // Upload remaining buffer as last part
        if !self.buffer.is_empty() {
            self.flush_part().await?;
        }

        self.storage
            .complete_multipart_upload(&self.key, &self.upload_id, self.parts)
            .await
            .map_err(|e| anyhow!("Failed to complete upload: {}", e))
    }

    pub async fn abort(&self) -> Result<()> {
        self.storage
            .abort_multipart_upload(&self.key, &self.upload_id)
            .await
            .map_err(|e| anyhow!("Failed to abort upload: {}", e))
    }
}

pub async fn stream_to_s3(
    storage: &StorageService,
    mut field: Field<'_>,
    key: String,
) -> Result<String> {
    let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

    // Validate request mime
    if !content_type.starts_with("video/") && !content_type.starts_with("image/") {
        return Err(anyhow!("Invalid content type: only video/* and image/* allowed"));
    }

    let mut uploader = MultipartUploader::new(storage, key.clone(), &content_type).await?;

    while let Some(chunk) = field.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => {
                error!("Stream error: {}", e);
                uploader.abort().await?;
                return Err(anyhow!("Stream interrupted"));
            }
        };

        if let Err(e) = uploader.write_chunk(chunk).await {
            error!("Upload error: {}", e);
            uploader.abort().await?;
            return Err(e);
        }
    }

    uploader.finish().await
}

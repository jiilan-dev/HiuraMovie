use aws_sdk_s3::{Client, config::Region, config::Credentials, config::BehaviorVersion};
use aws_sdk_s3::config::Builder;
use tracing::info;
use tokio::io::AsyncWriteExt;

#[derive(Clone)]
pub struct StorageService {
    pub client: Client,
    pub bucket: String,
}

impl StorageService {
    pub async fn new(
        endpoint: &str, 
        bucket: &str, 
        access_key: &str, 
        secret_key: &str
    ) -> Self {
        let credentials = Credentials::new(access_key, secret_key, None, None, "static");
        
        let config = Builder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new("us-east-1"))
            .endpoint_url(endpoint)
            .credentials_provider(credentials)
            .force_path_style(true) // Required for MinIO
            .build();

        let client = Client::from_conf(config);

        info!("✅ Connected to S3 (MinIO)");
        
        let storage = Self {
            client,
            bucket: bucket.to_string(),
        };

        // Auto-create default bucket
        if let Err(e) = storage.ensure_bucket_exists(bucket).await {
            tracing::warn!("Failed to ensure bucket '{}' exists: {}", bucket, e);
        }

        // We can't access other buckets from arguments here easily unless we pass them or
        // we rely on the caller to call ensure_bucket_exists.
        // For now, let's just return storage and let main.rs call ensure for others, 
        // OR we change the signature of new() to take a list of buckets?
        // But main.rs calls this. Let's keep it simple and just return 'storage'.
        // Wait, the user wants "auto create".
        // Let's modify the signature of new to take additional buckets?
        // Or better: Let's handle this in main.rs where we have full config access.
        
        storage
    }

    /// Ensure a specific bucket exists, create it if not
    pub async fn ensure_bucket_exists(&self, bucket_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Check if bucket exists
        let exists = self.client
            .head_bucket()
            .bucket(bucket_name)
            .send()
            .await;

        match exists {
            Ok(_) => {
                info!("✅ Bucket '{}' exists", bucket_name);
                Ok(())
            }
            Err(_) => {
                // Bucket doesn't exist, create it
                info!("🪣 Creating bucket '{}'...", bucket_name);
                self.client
                    .create_bucket()
                    .bucket(bucket_name)
                    .send()
                    .await?;
                info!("✅ Bucket '{}' created successfully", bucket_name);
                Ok(())
            }
        }
    }

    pub async fn create_multipart_upload(&self, key: &str, content_type: &str) -> Result<String, aws_sdk_s3::Error> {
        let result = self
            .client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .send()
            .await?;

        Ok(result.upload_id.unwrap())
    }

    pub async fn upload_part(
        &self,
        key: &str,
        upload_id: &str,
        part_number: i32,
        body: bytes::Bytes,
    ) -> Result<aws_sdk_s3::types::CompletedPart, aws_sdk_s3::Error> {
        let result = self
            .client
            .upload_part()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .part_number(part_number)
            .body(aws_sdk_s3::primitives::ByteStream::from(body))
            .send()
            .await?;

        Ok(aws_sdk_s3::types::CompletedPart::builder()
            .e_tag(result.e_tag.unwrap())
            .part_number(part_number)
            .build())
    }

    pub async fn complete_multipart_upload(
        &self,
        key: &str,
        upload_id: &str,
        parts: Vec<aws_sdk_s3::types::CompletedPart>,
    ) -> Result<String, aws_sdk_s3::Error> {
        let completed_multipart_upload = aws_sdk_s3::types::CompletedMultipartUpload::builder()
            .set_parts(Some(parts))
            .build();

        self.client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .multipart_upload(completed_multipart_upload)
            .send()
            .await?;

        Ok(format!("{}/{}", self.bucket, key))
    }

    pub async fn abort_multipart_upload(
        &self,
        key: &str,
        upload_id: &str,
    ) -> Result<(), aws_sdk_s3::Error> {
        self.client
            .abort_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>, aws_sdk_s3::Error> {
        let result = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        let data = result.body.collect().await.map(|d| d.into_bytes().to_vec()).unwrap_or_default();
        Ok(data)
    }

    pub async fn download_file(&self, key: &str, file_path: &str) -> Result<(), anyhow::Error> {
        let mut result = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("S3 GetObject Error: {}", e))?;

        let mut file = tokio::fs::File::create(file_path).await
            .map_err(|e| anyhow::anyhow!("Failed to create file: {}", e))?;
        
        while let Some(chunk) = result.body.next().await {
            let data = chunk.map_err(|e| anyhow::anyhow!("S3 Stream Error: {}", e))?;
            file.write_all(&data).await
                .map_err(|e| anyhow::anyhow!("Write Error: {}", e))?;
        }
        
        file.flush().await?;
        Ok(())
    }
}

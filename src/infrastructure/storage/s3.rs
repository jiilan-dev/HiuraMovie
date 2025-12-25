use aws_sdk_s3::{Client, config::Region, config::Credentials, config::BehaviorVersion};
use aws_sdk_s3::config::Builder;
use tracing::info;

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
        
        Self {
            client,
            bucket: bucket.to_string(),
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
}

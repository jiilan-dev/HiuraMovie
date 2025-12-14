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
}

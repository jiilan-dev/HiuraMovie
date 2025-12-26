use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscodeJob {
    pub content_id: Uuid,
    pub content_type: String, // "movie" or "episode"
    pub s3_key: String,
}

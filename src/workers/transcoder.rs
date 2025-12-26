use crate::modules::content::events::TranscodeJob;
use crate::state::AppState;
use futures_util::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use std::process::Command;
use tracing::{error, info};
use std::fs;


pub async fn start_transcoder_worker(state: AppState) {
    info!("🎥 Starting Transcoder Worker...");

    let channel = state.queue.get_channel().await;
    let channel_guard = channel.lock().await;

    let queue_name = "transcoding_tasks";

    // Declare queue
    let _queue = channel_guard
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable: true,
                ..QueueDeclareOptions::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare queue");

    // Create consumer
    let mut consumer = channel_guard
        .basic_consume(
            queue_name,
            "transcoder_worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to create consumer");
        
    // Drop lock to allow other operations on channel if needed, 
    // though for basic_consume we usually hold it or clone channel.
    // Actually, consumer stream is independent.
    drop(channel_guard);

    info!("🎥 Transcoder Worker listening on '{}'", queue_name);

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let payload = delivery.data.clone();
            
            // Spawn task to process each message concurrently? 
            // Or sequentially to avoid overloading CPU with ffmpeg?
            // Sequentially is safer for transcoding.
            
            info!("📦 Received transcoding job");

            match serde_json::from_slice::<TranscodeJob>(&payload) {
                Ok(job) => {
                    if let Err(e) = process_job(&state, &job).await {
                        error!("❌ Failed to process job {:?}: {}", job, e);
                        // Negative Ack? Or just log and ack? 
                        // For now, let's ack to avoid infinite loop of death, or nack with requeue=false.
                        // Ideally dead letter queue.
                    } else {
                        info!("✅ Job completed successfully: {:?}", job);
                    }
                }
                Err(e) => {
                    error!("❌ Failed to parse job: {}", e);
                }
            }

            // Ack message
            if let Err(e) = delivery
                .ack(BasicAckOptions::default())
                .await
            {
                error!("Failed to ack message: {}", e);
            }
        }
    }
}

async fn process_job(state: &AppState, job: &TranscodeJob) -> anyhow::Result<()> {
    info!("Processing job: {:?}", job);
    
    // 1. Download file
    info!("⬇️ Downloading file from S3: {}", job.s3_key);
    let file_data = state.storage.get_object(&job.s3_key).await
        .map_err(|e| anyhow::anyhow!("Failed to download from S3: {}", e))?;
    
    info!("⬇️ Downloaded {} bytes", file_data.len());
        
    let input_path = format!("/tmp/{}_input.mkv", job.content_id);
    fs::write(&input_path, &file_data)?;
    
    // 2. Transcode to MP4
    let output_mp4 = format!("/tmp/{}_output.mp4", job.content_id);
    let status = Command::new("ffmpeg")
        .args(&[
            "-i", &input_path,
            "-c:v", "libx264",
            "-preset", "fast",
            "-c:a", "aac",
            "-strict", "experimental",
            "-y", // overwrite
            &output_mp4
        ])
        .status()?;
        
    if !status.success() {
        return Err(anyhow::anyhow!("FFmpeg failed to transcode"));
    }
    
    // 3. Extract Subtitle (VTT)
    let output_vtt = format!("/tmp/{}_output.vtt", job.content_id);
    // Try extract first subtitle track
    // If fail (no subtitle), we just continue without subtitle
    let sub_status = Command::new("ffmpeg")
        .args(&[
            "-i", &input_path,
            "-map", "0:s:0",
            "-y",
            &output_vtt
        ])
        .status();
        
    let has_subtitle = sub_status.map(|s| s.success()).unwrap_or(false);
    
    // 4. Upload MP4
    let mp4_key = format!("processed/{}.mp4", job.content_id);
    let mp4_data = fs::read(&output_mp4)?;
    
    // Use low-level S3 upload or add upload_file to StorageService?
    // StorageService has upload_part logic but not simple put_object wrapper exposed widely?
    // It has `create_multipart_upload` etc. in common/upload.rs but that's for streaming from Axum.
    // I should probably add `put_object` to StorageService for simple byte definition.
    // For now I'll use `storage.client.put_object` directly if pub, or add helper.
    // StorageService fields are pub.
    
    state.storage.client.put_object()
        .bucket(&state.storage.bucket)
        .key(&mp4_key)
        .body(aws_sdk_s3::primitives::ByteStream::from(mp4_data))
        .content_type("video/mp4")
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to upload MP4: {}", e))?;
        
    // 5. Upload VTT (if exists)
    let mut vtt_key_opt: Option<String> = None;
    if has_subtitle {
        let vtt_key = format!("subtitles/{}.vtt", job.content_id);
        let vtt_data = fs::read(&output_vtt)?;
        
        state.storage.client.put_object()
            .bucket(&state.storage.bucket)
            .key(&vtt_key)
            .body(aws_sdk_s3::primitives::ByteStream::from(vtt_data))
            .content_type("text/vtt")
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to upload VTT: {}", e))?;
            
        vtt_key_opt = Some(vtt_key);
    }
    
    // 6. Update DB
    // We need to call Repositories. But repositories need generic DbPool.
    // Handlers use ContentService -> ContentRepository.
    // Workers should probably use ContentRepository directly or Service.
    // But they take AppState. 
    // Let's instantiate ContentRepository here locally or add it to AppState?
    // AppState has db.
    
    let db = &state.db;
    
    // We can't easily instantiate Repository struct if it's not clonable or has lifecycle.
    // But ContentRepository usually just holds logic.
    // Ideally update SQL here or use Repository methods if they are static/stateless.
    // ContentRepository in this codebase seems to be just methods on struct, no specific state other than maybe (?).
    // Let's check ContentRepository definition. 
    // Assuming we can use SQLX directly for simplicity here.
    
    if job.content_type == "episode" {
        sqlx::query!(
            "UPDATE episodes SET video_url = $1, subtitle_url = $2, status = 'READY', updated_at = NOW() WHERE id = $3",
            mp4_key,
            vtt_key_opt,
            job.content_id
        )
        .execute(db)
        .await
        .map_err(|e| anyhow::anyhow!("DB Error: {}", e))?;
    } else {
        // Movie
         sqlx::query!(
            "UPDATE movies SET video_url = $1, subtitle_url = $2, status = 'READY', updated_at = NOW() WHERE id = $3",
            mp4_key,
            vtt_key_opt,
            job.content_id
        )
        .execute(db)
        .await
        .map_err(|e| anyhow::anyhow!("DB Error: {}", e))?;
    }
    
    // 7. Cleanup
    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_mp4);
    if has_subtitle {
        let _ = fs::remove_file(output_vtt);
    }
    
    Ok(())
}

use crate::modules::content::events::TranscodeJob;
use crate::state::AppState;
use futures_util::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use tokio::io::{AsyncBufReadExt, BufReader};
use std::process::Stdio;
use tracing::{error, info, warn};
use redis::AsyncCommands;
use std::fs;


pub async fn start_transcoder_worker(state: AppState) {
    info!("🎥 Starting Transcoder Worker...");

    let queue_name = "transcoding_tasks";

    loop {
        let channel = state.queue.get_channel().await;
        let channel_guard = channel.lock().await;

        let _queue = match channel_guard
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..QueueDeclareOptions::default()
                },
                FieldTable::default(),
            )
            .await
        {
            Ok(queue) => queue,
            Err(e) => {
                error!("Failed to declare queue '{}': {}", queue_name, e);
                drop(channel_guard);
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        let mut consumer = match channel_guard
            .basic_consume(
                queue_name,
                "transcoder_worker",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
        {
            Ok(consumer) => consumer,
            Err(e) => {
                error!("Failed to create consumer: {}", e);
                drop(channel_guard);
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        drop(channel_guard);

        info!("🎥 Transcoder Worker listening on '{}'", queue_name);

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    let payload = delivery.data.clone();

                    info!("📦 Received transcoding job");

                    match serde_json::from_slice::<TranscodeJob>(&payload) {
                        Ok(job) => {
                            if let Err(e) = process_job(&state, &job).await {
                                error!("❌ Failed to process job {:?}: {}", job, e);
                            } else {
                                info!("✅ Job completed successfully: {:?}", job);
                            }
                        }
                        Err(e) => {
                            error!("❌ Failed to parse job: {}", e);
                        }
                    }

                    if let Err(e) = delivery
                        .ack(BasicAckOptions::default())
                        .await
                    {
                        error!("Failed to ack message: {}", e);
                    }
                }
                Err(e) => {
                    error!("Transcoder consumer error: {}", e);
                    break;
                }
            }
        }

        warn!("Transcoder consumer stopped, retrying in 2s...");
        sleep(Duration::from_secs(2)).await;
    }
}

async fn process_job(state: &AppState, job: &TranscodeJob) -> anyhow::Result<()> {
    info!("Processing job: {:?}", job);
    
    // 1. Download file
    // 1. Download file
    let input_path = format!("/tmp/{}_input.mkv", job.content_id);
    state.storage.download_file(&job.s3_key, &input_path).await
        .map_err(|e| anyhow::anyhow!("Failed to download from S3: {}", e))?;
    
    let progress_key = format!("transcode_progress:{}:{}", job.content_type, job.content_id);
    let mut redis_conn = match state.redis.get_conn().await {
        Ok(conn) => Some(conn),
        Err(e) => {
            warn!("Failed to connect to Redis for progress tracking: {}", e);
            None
        }
    };

    set_transcode_progress(redis_conn.as_mut(), &progress_key, 0).await;

    // 2. Transcode to MP4
    let output_mp4 = format!("/tmp/{}_output.mp4", job.content_id);
    let duration_ms = get_media_duration_ms(&input_path).await;
    transcode_with_progress(
        &input_path,
        &output_mp4,
        duration_ms,
        &mut redis_conn,
        &progress_key,
    ).await?;
    
    // 3. Extract Subtitle (VTT)
    let output_vtt = format!("/tmp/{}_output.vtt", job.content_id);
    let has_subtitle = if has_subtitle_stream(&input_path).await {
        let sub_status = Command::new("ffmpeg")
            .args(&[
                "-hide_banner",
                "-loglevel", "error",
                "-i", &input_path,
                "-threads", "1",
                "-map", "0:s:0",
                "-y",
                &output_vtt
            ])
            .status()
            .await;

        sub_status.map(|s| s.success()).unwrap_or(false)
    } else {
        false
    };
    
    // 4. Upload MP4
    let mp4_key = format!("processed/{}.mp4", job.content_id);
    // let mp4_data = fs::read(&output_mp4)?; // Removed to save RAM
    
    let body = aws_sdk_s3::primitives::ByteStream::from_path(std::path::Path::new(&output_mp4)).await
        .map_err(|e| anyhow::anyhow!("Failed to read MP4 file: {}", e))?;
    
    state.storage.client.put_object()
        .bucket(&state.storage.bucket)
        .key(&mp4_key)
        .body(body)
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

    set_transcode_progress(redis_conn.as_mut(), &progress_key, 100).await;
    
    Ok(())
}

async fn has_subtitle_stream(input_path: &str) -> bool {
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-select_streams", "s:0",
            "-show_entries", "stream=index",
            "-of", "csv=p=0",
            input_path,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await;

    match output {
        Ok(out) => !out.stdout.is_empty(),
        Err(e) => {
            warn!("ffprobe not available ({}), skipping subtitle extraction", e);
            false
        }
    }
}

async fn get_media_duration_ms(input_path: &str) -> Option<u64> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=nk=1:nw=1",
            input_path,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let duration_str = String::from_utf8_lossy(&output.stdout);
    let duration_secs = duration_str.trim().parse::<f64>().ok()?;
    Some((duration_secs * 1000.0) as u64)
}

async fn transcode_with_progress(
    input_path: &str,
    output_mp4: &str,
    duration_ms: Option<u64>,
    redis_conn: &mut Option<redis::aio::MultiplexedConnection>,
    progress_key: &str,
) -> anyhow::Result<()> {
    let mut child = Command::new("ffmpeg")
        .args(&[
            "-hide_banner",
            "-loglevel", "error",
            "-i", input_path,
            "-threads", "1",
            "-c:v", "libx264",
            "-preset", "ultrafast",
            "-c:a", "aac",
            "-strict", "experimental",
            "-y",
            "-progress", "pipe:1",
            "-nostats",
            output_mp4
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to read ffmpeg progress"))?;
    let mut reader = BufReader::new(stdout).lines();
    let mut last_percent: u8 = 0;

    while let Some(line) = reader.next_line().await? {
        if let Some(out_time_ms) = line.strip_prefix("out_time_ms=") {
            if let Some(total_ms) = duration_ms {
                if total_ms > 0 {
                    let raw = out_time_ms.trim().parse::<u64>().unwrap_or(0);
                    let mut percent = ((raw as f64 / total_ms as f64) * 100.0).round() as u8;
                    if percent > 99 {
                        percent = 99;
                    }
                    if percent != last_percent {
                        last_percent = percent;
                        set_transcode_progress(redis_conn.as_mut(), progress_key, percent).await;
                    }
                }
            }
        }
    }

    let status = child.wait().await?;
    if !status.success() {
        return Err(anyhow::anyhow!("FFmpeg failed to transcode"));
    }

    Ok(())
}

async fn set_transcode_progress(
    redis_conn: Option<&mut redis::aio::MultiplexedConnection>,
    key: &str,
    percent: u8,
) {
    let Some(conn) = redis_conn else { return };
    let ttl_seconds = 60 * 60; // 1 hour
    let result: Result<(), redis::RedisError> = conn.set_ex(key, percent, ttl_seconds).await;
    if let Err(e) = result {
        warn!("Failed to update transcode progress: {}", e);
    }
}

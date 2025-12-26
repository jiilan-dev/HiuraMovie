use crate::infrastructure::storage::s3::StorageService;
use crate::modules::content::events::TranscodeJob;
use crate::state::AppState;
use bytes::Bytes;
use futures_util::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use std::process::Stdio;
use tracing::{error, info, warn};
use redis::AsyncCommands;
use std::fs;
use tokio::fs as tokio_fs;


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
                if let Err(err) = state.queue.reconnect().await {
                    warn!("Failed to reconnect RabbitMQ after declare error: {}", err);
                }
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
                if let Err(err) = state.queue.reconnect().await {
                    warn!("Failed to reconnect RabbitMQ after consume error: {}", err);
                }
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
        if let Err(err) = state.queue.reconnect().await {
            warn!("Failed to reconnect RabbitMQ after consumer stop: {}", err);
        }
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
    upload_file_multipart_with_retry(
        &state.storage,
        &mp4_key,
        &output_mp4,
        "video/mp4",
    )
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
            "-threads", "0",
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

async fn upload_file_multipart_with_retry(
    storage: &StorageService,
    key: &str,
    file_path: &str,
    content_type: &str,
) -> anyhow::Result<()> {
    let mut attempt = 0;
    let max_retries = 3;

    loop {
        attempt += 1;
        match upload_file_multipart(storage, key, file_path, content_type).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                if attempt >= max_retries {
                    return Err(e);
                }
                warn!(
                    "Upload failed for '{}' (attempt {}/{}): {:?}",
                    key,
                    attempt,
                    max_retries,
                    e
                );
                sleep(Duration::from_millis(500 * attempt as u64)).await;
            }
        }
    }
}

async fn upload_file_simple(
    storage: &StorageService,
    key: &str,
    file_path: &str,
    content_type: &str,
    content_length: u64,
) -> anyhow::Result<()> {
    let body = aws_sdk_s3::primitives::ByteStream::from_path(std::path::Path::new(file_path))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read output file: {}", e))?;

    storage
        .client
        .put_object()
        .bucket(&storage.bucket)
        .key(key)
        .body(body)
        .content_type(content_type)
        .content_length(content_length as i64)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to upload file: {:?}", e))?;

    Ok(())
}

async fn upload_file_multipart(
    storage: &StorageService,
    key: &str,
    file_path: &str,
    content_type: &str,
) -> anyhow::Result<()> {
    const PART_SIZE: usize = 6 * 1024 * 1024;
    const MIN_PART_SIZE: u64 = 5 * 1024 * 1024;

    let meta = tokio_fs::metadata(file_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read output metadata: {}", e))?;
    if meta.len() == 0 {
        return Err(anyhow::anyhow!("Output file is empty"));
    }
    if meta.len() < MIN_PART_SIZE {
        return upload_file_simple(storage, key, file_path, content_type, meta.len()).await;
    }

    let upload_id = storage
        .create_multipart_upload(key, content_type)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initiate multipart upload: {}", e))?;

    let mut parts = Vec::new();
    let mut part_number: i32 = 1;
    let mut file = tokio_fs::File::open(file_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to open output file: {}", e))?;
    let mut chunk = Vec::with_capacity(PART_SIZE);
    let mut read_buf = [0u8; 64 * 1024];

    let result: anyhow::Result<()> = async {
        loop {
            let read = file
                .read(&mut read_buf)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read output file: {}", e))?;

            if read == 0 {
                break;
            }

            chunk.extend_from_slice(&read_buf[..read]);

            if chunk.len() >= PART_SIZE {
                let body = Bytes::copy_from_slice(&chunk);
                let part = storage
                    .upload_part(key, &upload_id, part_number, body)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to upload part {}: {}", part_number, e))?;

                parts.push(part);
                part_number += 1;
                chunk.clear();
            }
        }

        if !chunk.is_empty() {
            let body = Bytes::copy_from_slice(&chunk);
            let part = storage
                .upload_part(key, &upload_id, part_number, body)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to upload part {}: {}", part_number, e))?;

            parts.push(part);
        }
        Ok(())
    }
    .await;

    if let Err(err) = result {
        let _ = storage.abort_multipart_upload(key, &upload_id).await;
        return Err(err);
    }

    storage
        .complete_multipart_upload(key, &upload_id, parts)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to complete multipart upload: {}", e))?;

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

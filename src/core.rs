use std::{fs::File, io::{Write, Seek, SeekFrom}, path::Path, sync::{Arc, Mutex}, time::Instant};
use reqwest::{Client, Response, StatusCode};
use tokio::{sync::Mutex as AsyncMutex, task};
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;
use tracing::{info, error};

pub struct DownloadEngine {
    client: Client,
    chunk_size: u64,
}

impl DownloadEngine {
    pub fn new(chunk_size: u64) -> Self {
        let _ = tracing_subscriber::fmt::try_init(); // Initialize logger
        Self {
            client: Client::new(),
            chunk_size,
        }
    }

    pub async fn download(&self, url: &str, target: &str) -> Result<(), Box<dyn std::error::Error>> {
        let total_size = self.get_content_length(url).await?;
        if total_size == 0 {
            error!("Failed to fetch content length, falling back to streaming");
            return self.download_fallback(url, target).await;
        }
        info!("Total file size: {} bytes", total_size);
        
        let existing_size = Self::get_existing_file_size(target)?;
        let progress = ProgressBar::new(total_size);
        progress.set_style(ProgressStyle::default_bar());
        progress.set_message("Downloading");
        progress.set_position(existing_size);

        let path = Path::new(target);
        let file = Arc::new(AsyncMutex::new(File::options().create(true).append(true).open(path)?));
        let start_time = Instant::now();
        let num_chunks = (total_size - existing_size + self.chunk_size - 1) / self.chunk_size;

        let mut handles = vec![];
        for i in 0..num_chunks {
            let client = self.client.clone();
            let file = file.clone();
            let progress = progress.clone();
            let url = url.to_string();
            let start = existing_size + i * self.chunk_size;
            let end = (start + self.chunk_size - 1).min(total_size - 1);
            
            let handle = task::spawn(async move {
                info!("Downloading chunk: {} - {}", start, end);
                let response = client.get(&url).header("Range", format!("bytes={}-{}", start, end)).send().await;
                if let Err(e) = response {
                    error!("Request failed for range {}-{}: {:?}", start, end, e);
                    return None;
                }
                let response = response.unwrap();
                if response.status() != StatusCode::PARTIAL_CONTENT && response.status() != StatusCode::OK {
                    error!("Server does not support partial download. Status: {:?}", response.status());
                    return None;
                }
                let mut stream = response.bytes_stream();
                let mut downloaded = 0;

                while let Some(chunk) = stream.next().await {
                    let chunk = match chunk {
                        Ok(c) => c,
                        Err(e) => {
                            error!("Error while downloading chunk {}-{}: {:?}", start, end, e);
                            return None;
                        }
                    };
                    let mut file_lock = file.lock().await;
                    file_lock.seek(SeekFrom::Start(start + downloaded)).ok()?;
                    file_lock.write_all(&chunk).ok()?;
                    downloaded += chunk.len() as u64;
                    progress.inc(chunk.len() as u64);
                    
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let speed = if elapsed > 0.0 { Self::format_speed(progress.position() as f64 / elapsed) } else { "0 B/s".to_string() };
                    progress.set_message(format!("Speed: {}", speed));
                }
                info!("Chunk {}-{} downloaded successfully", start, end);
                Some(())
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.ok();
        }

        progress.finish_with_message("Download complete");
        info!("Download completed successfully!");
        Ok(())
    }

    async fn get_content_length(&self, url: &str) -> Result<u64, Box<dyn std::error::Error>> {
        let response = self.client.head(url).send().await;
        match response {
            Ok(resp) if resp.status().is_success() => {
                if let Some(length) = resp.content_length() {
                    return Ok(length);
                }
            }
            _ => {
                error!("HEAD request failed, trying GET request to determine file size");
                let resp = self.client.get(url).send().await?;
                if let Some(length) = resp.content_length() {
                    return Ok(length);
                }
            }
        }
        Ok(0)
    }

    async fn download_fallback(&self, url: &str, target: &str) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.get(url).send().await?;
        let mut file = File::create(target)?;
        let mut stream = response.bytes_stream();
        let progress = ProgressBar::new_spinner();
        progress.set_message("Downloading");
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
            progress.inc(chunk.len() as u64);
        }
        progress.finish_with_message("Download complete");
        info!("Download completed successfully!");
        Ok(())
    }
    
    fn get_existing_file_size(target: &str) -> std::io::Result<u64> {
        let path = Path::new(target);
        if path.exists() {
            let metadata = std::fs::metadata(path)?;
            Ok(metadata.len())
        } else {
            Ok(0)
        }
    }

    fn format_speed(speed: f64) -> String {
        if speed > 1_048_576.0 {
            format!("{:.2} MB/s", speed / 1_048_576.0)
        } else if speed > 1024.0 {
            format!("{:.2} KB/s", speed / 1024.0)
        } else {
            format!("{:.2} B/s", speed)
        }
    }
}

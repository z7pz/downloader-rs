use std::{cmp::min, fs::File, io::Write, path::Path, process::exit};

use clap::{Parser, command};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Request};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(name = "downloader")]
#[command(version = "0.1.0")]
#[command(about = "blazingly fast donwloader!")]
pub struct Args {
    #[clap(short, long)]
    pub url: String,

    #[clap(short, long)]
    pub target: String,
}

fn create_file(path: &str) -> File {
    let path = Path::new(path);
    if path.exists() {
        println!("File already exists.");
        exit(1)
    }
    File::create(path).expect("Failed to create file")
}

async fn download_file(url: &str, target: &str) {
    let client = Client::new();
    let req = client.get(url).send().await.expect("Failed to connect to");
    let download_size = req.content_length().unwrap_or_default();
    let progress = ProgressBar::new(download_size);
    let progress_style = ProgressStyle::default_bar();
    progress.set_style(progress_style);
    progress.set_message(format!("Downloading {}", url));
    let mut downloaded = 0;
    let mut file = create_file(target);
    let mut stream = req.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item.expect("Error while downloading file");
        file.write(&chunk).expect("Error while writing to file");
        downloaded = min(downloaded + (chunk.len() as u64), download_size);
        progress.set_position(downloaded);
    }

    progress.finish_with_message(format!("  Downloaded {} to {}", url, target));
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    download_file(&args.url, &args.target).await;
}

use clap::{Parser, command};
use core::DownloadEngine;
use tokio;

mod core;

#[derive(Parser, Debug)]
#[command(name = "download-cli")]
#[command(version = "0.1.0")]
#[command(about = "CLI client for download engine")]
pub struct Args {
    #[clap(short, long)]
    pub url: String,

    #[clap(short, long)]
    pub target: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let engine = DownloadEngine::new(1048576 * 100);

    match engine.download(&args.url, &args.target).await {
        Ok(_) => println!("Download completed successfully!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
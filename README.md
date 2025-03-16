# Downloader

## Overview
Downloader is a blazingly fast file downloader written in Rust. It efficiently downloads files from a given URL and saves them to a specified target location. The tool provides a progress bar to track the download process.

## Features
- Fast and efficient file downloading
- CLI support using `clap`
- Progress tracking with `indicatif`
- Asynchronous execution using `tokio`

## Prerequisites
To build and run this project, you need:
- Rust (latest stable version)
- Cargo package manager

## Installation
Clone the repository and navigate into the project directory:
```sh
git clone https://github.com/yourusername/downloader.git
cd downloader
```
Build the project using Cargo:
```sh
cargo build --release
```

## Usage
Run the downloader with the required arguments:
```sh
./target/release/downloader -u <URL> -t <TARGET_PATH>
```
### Example:
```sh
./target/release/downloader -u https://example.com/file.zip -t ./file.zip
```

## Dependencies
The project uses the following dependencies:
- `clap` for command-line argument parsing
- `reqwest` for making HTTP requests
- `indicatif` for progress tracking
- `tokio` for asynchronous execution
- `tracing` for logging

## License
This project is licensed under the MIT License. See the `LICENSE` file for details.

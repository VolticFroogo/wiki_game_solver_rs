mod parse_sql_file;
mod link_mapping;
mod file_download;

use crate::link_mapping::generate_and_save_link_mapping;
use crate::file_download::download_wikipedia_db_dumps;
use std::error::Error;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Download latest SQL dumps and save link mapping daily
    loop {
        println!("Downloading Wikipedia DB dumps...");
        let download_result = download_wikipedia_db_dumps().await;
        if let Err(e) = download_result {
            eprintln!("Error downloading Wikipedia DB dumps: {}", e);
            println!("Retrying in 1 hour...");
            tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await; // 1 hour
            continue;
        }

        println!("Generating link mapping...");
        let link_mapping_result = generate_and_save_link_mapping();
        if let Err(e) = link_mapping_result {
            eprintln!("Error generating link mapping: {}", e);
            println!("Retrying in 1 hour...");
            tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await; // 1 hour
            continue;
        }

        println!("Cleaning up downloaded files...");
        let cleanup_result = file_download::delete_wikipedia_db_dumps();
        if let Err(e) = cleanup_result {
            eprintln!("Error cleaning up downloaded files: {}", e);
        }

        println!("Sleeping for 24 hours...");
        tokio::time::sleep(std::time::Duration::from_secs(24 * 60 * 60)).await; // 24 hours
    }
}


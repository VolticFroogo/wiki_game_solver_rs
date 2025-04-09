use std::fs;
use async_compression::tokio::bufread::GzipDecoder;
use futures_util::StreamExt;
use reqwest::Client;
use std::fs::File;
use std::io::{BufWriter, Error, ErrorKind, Write};
use std::path::Path;
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;

const WIKIPEDIA_DUMP_URL: &str = "https://dumps.wikimedia.org/enwiki/latest/";

pub(crate) async fn download_wikipedia_db_dumps() -> crate::Result<()> {
    // Wikipedia dumps only allows two simultaneous connections
    // Download (page, linktarget) and (pagelinks) files in parallel
    // Since pagelinks is much larger it's no slower to block the first two anyway
    let (task_first, task_second) = tokio::join!(
        async {
            download_and_decompress(
                format!("{WIKIPEDIA_DUMP_URL}enwiki-latest-page.sql.gz").as_str(),
                "sql/enwiki-latest-page.sql",
            ).await?;

            download_and_decompress(
                format!("{WIKIPEDIA_DUMP_URL}enwiki-latest-linktarget.sql.gz").as_str(),
                "sql/enwiki-latest-linktarget.sql",
            ).await?;

            crate::Result::Ok(())
        },
        async {
            download_and_decompress(
                format!("{WIKIPEDIA_DUMP_URL}enwiki-latest-pagelinks.sql.gz").as_str(),
                "sql/enwiki-latest-pagelinks.sql",
            ).await?;

            crate::Result::Ok(())
        },
    );

    task_first?;
    task_second?;

    Ok(())
}

pub(crate) fn delete_wikipedia_db_dumps() -> crate::Result<()> {
    let files = [
        "sql/enwiki-latest-page.sql",
        "sql/enwiki-latest-linktarget.sql",
        "sql/enwiki-latest-pagelinks.sql",
    ];

    for file in files {
        if fs::exists(file)? {
            fs::remove_file(file)?;
            println!("Deleted {}", file);
        }
    }

    Ok(())
}

async fn download_and_decompress(url: &str, output_path: &str) -> crate::Result<()> {
    println!("Downloading {} to {}", url, output_path);
    let start = std::time::Instant::now();

    let client = Client::new();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to download file {}: HTTP {}", url, response.status()).into());
    }

    fs::create_dir_all(Path::new(output_path).parent().unwrap())?;
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    let stream_reader = StreamReader::new(
        response
            .bytes_stream()
            .map(|x| x.map_err(|e| Error::new(ErrorKind::Other, e.to_string()))),
    );

    let mut decoder = GzipDecoder::new(stream_reader);

    let mut buffer = vec![0; 64 * 1024]; // 64 KiB buffer
    loop {
        let bytes_read = decoder.read(&mut buffer).await?;

        if bytes_read == 0 {
            break; // EOF
        }

        writer.write_all(&buffer[..bytes_read])?;
    }

    println!("File downloaded and decompressed successfully to {} in {}s", output_path, start.elapsed().as_secs());
    Ok(())
}
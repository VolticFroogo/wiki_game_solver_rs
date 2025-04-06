use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::time::Instant;
use regex::{Captures, Regex};
use crate::Result;

const CHUNK_SIZE: usize = 64 * 1024;

pub fn parse_sql_file<F: FnMut(&Captures)>(file_name: &str, regex: Regex, mut func: F) -> Result<()> {
    let start = Instant::now();

    let mut file = File::open(file_name)?;
    let file_size = file.metadata()?.len() as usize;
    let mut buffer = [0u8; CHUNK_SIZE];
    let mut total_bytes_read = 0usize;

    file_seek_to_values(&mut file, &mut buffer)?;

    while let Ok(bytes_read) = file.read(&mut buffer) {
        total_bytes_read += bytes_read;
        print_progress(file_name, file_size, &total_bytes_read);

        let bytes_as_string = String::from_utf8_lossy(&buffer[..bytes_read]);
        let captures: Vec<Captures> = regex.captures_iter(&bytes_as_string).collect();
        let _ = &captures.iter().for_each(&mut func);

        if bytes_read < CHUNK_SIZE {
            break; // EOF
        }

        file_seek_to_end_of_last_match(&mut file, captures.last())?;
    }

    println!("Finished reading {} in {}s", file_name, start.elapsed().as_secs());
    Ok(())
}

// Seek to the first occurrence of "VALUES" in the file
// This prevents finding matches outside the VALUES section
fn file_seek_to_values(file: &mut File, mut buffer: &mut [u8]) -> Result<()> {
    let bytes_read = file.read(&mut buffer)?;
    let bytes_as_string = String::from_utf8_lossy(&buffer[..bytes_read]);

    if let Some(pos) = bytes_as_string.find("VALUES") {
        let offset_pos = pos + "VALUES ".len();
        file.seek(SeekFrom::Start(offset_pos as u64))?;
    } else {
        return Err("Could not find 'VALUES' in the file".into());
    }

    Ok(())
}

// To prevent missing matches in between chunks, we seek back to the end of the last match
fn file_seek_to_end_of_last_match(file: &mut File, last_capture: Option<&Captures>) -> Result<()> {
    if let Some(cap) = last_capture {
        let last = cap.iter().last().unwrap().unwrap();
        let offset = last.end() as i64 - CHUNK_SIZE as i64;
        file.seek_relative(offset)?;
    }

    Ok(())
}

fn print_progress(file_name: &str, file_size: usize, total_bytes_read: &usize) {
    if total_bytes_read % (1024 * CHUNK_SIZE) == 0 {
        println!("Read {}/{} MiB ({}%) of {}",
                 total_bytes_read / (1024 * 1024),
                 file_size / (1024 * 1024),
                 (total_bytes_read * 100) / file_size,
                 file_name);
    }
}

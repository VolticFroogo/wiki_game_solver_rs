use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use regex::{Captures, Regex};
use crate::Result;

const CHUNK_SIZE: usize = 64 * 1024;

pub fn parse_sql_file<F: FnMut(&Captures)>(file_name: &str, regex: Regex, mut func: F) -> Result<()> {
    let mut file = File::open(file_name)?;
    let mut buffer = [0u8; CHUNK_SIZE];

    let bytes_read = file.read(&mut buffer)?;
    let bytes_as_string = String::from_utf8_lossy(&buffer[..bytes_read]);
    if let Some(pos) = bytes_as_string.find("VALUES") {
        let offset_pos = pos + "VALUES ".len();
        file.seek(SeekFrom::Start(offset_pos as u64))?;
    } else {
        return Err("Could not find 'VALUES' in the file".into());
    }

    while let Ok(bytes_read) = file.read(&mut buffer) {
        if bytes_read == 0 {
            break; // EOF
        }

        let bytes_as_string = String::from_utf8_lossy(&buffer[..bytes_read]);
        let captures: Vec<Captures> = regex.captures_iter(&bytes_as_string).collect();
        let _ = &captures.iter().for_each(&mut func);

        if bytes_read < CHUNK_SIZE {
            break; // EOF
        }

        if let Some(cap) = captures.last() {
            let last = cap.iter().last().unwrap().unwrap();
            let offset = last.end() as i64 - CHUNK_SIZE as i64 + 3;
            file.seek_relative(offset)?;
        }
    }

    Ok(())
}

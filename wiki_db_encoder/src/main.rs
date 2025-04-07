mod parse_sql_file;

use crate::parse_sql_file::parse_sql_file;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

// Parse the SQL files and extract a map of page ID to vector of page IDs it links to
// TODO: explain logic and DB structure
fn main() -> Result<()> {
    let mut page: HashMap<String, u32> = HashMap::new();
    parse_sql_file(
        "enwiki-latest-page.sql",
        Regex::new(r"\((\d+),0,'((?:[^'\\]|\\.)*)',[\w\d.,']+\)").unwrap(),
        |cap| {
            let page_id = cap[1].parse::<u32>().unwrap();
            let title = cap[2].to_string();

            page.insert(title, page_id);
        })?;

    let mut link_target: HashMap<u32, u32> = HashMap::new();
    parse_sql_file(
        "enwiki-latest-linktarget.sql",
        Regex::new(r"\((\d+),0,'((?:[^'\\]|\\.)*)'\)").unwrap(),
        |cap| {
            let link_target_id = cap[1].parse::<u32>().unwrap();
            let title = &cap[2];

            if let Some(page_id) = page.get(title) {
                link_target.insert(link_target_id, *page_id);
            }
        })?;

    page.clear();

    let mut links: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut link_count = 0u32;
    parse_sql_file(
        "enwiki-latest-pagelinks.sql",
        Regex::new(r"\((\d+),0,(\d+)\)").unwrap(),
        |cap| {
            let from_page_id = cap[1].parse::<u32>().unwrap();
            let to_link_target_id = cap[2].parse::<u32>().unwrap();

            if let Some(to_page_id) = link_target.get(&to_link_target_id) {
                link_count += 1;
                links.entry(from_page_id).or_insert_with(Vec::new).push(*to_page_id);
            }
        })?;

    write_to_bincode_file(&links, &link_count)?;

    Ok(())
}

// Using Bincode, encode links to a file
fn write_to_bincode_file(links: &HashMap<u32, Vec<u32>>, link_count: &u32) -> Result<()> {
    println!("Found {} links from {} pages, writing bincode to links.bin", link_count, links.len());

    let start = Instant::now();
    let file = File::create("links.bin")?;
    let mut buffered_file = BufWriter::new(file);
    bincode::encode_into_std_write(&links, &mut buffered_file, bincode::config::standard())?;

    println!("Finished writing {} MiB to links.bin in {}s",
             buffered_file.get_ref().metadata()?.len() / (1024 * 1024),
             start.elapsed().as_secs());

    Ok(())
}
mod parse_sql_file;

use crate::parse_sql_file::parse_sql_file;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

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

                if let Some(from_page_vec) = links.get_mut(&from_page_id) {
                    from_page_vec.push(*to_page_id);
                } else {
                    links.insert(from_page_id, vec![*to_page_id]);
                }
            }
        })?;

    println!("Found {} links from {} pages, writing bincode to links.bin", link_count, links.len());

    // Using Bincode, encode links to a file
    let mut file = File::create("links.bin")?;
    bincode::encode_into_std_write(&links, &mut file, bincode::config::standard())?;

    println!("Finished writing {} MiB to links.bin", file.metadata()?.len() / (1024 * 1024));

    Ok(())
}
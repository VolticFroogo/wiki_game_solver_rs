mod parse_sql_file;

use crate::parse_sql_file::parse_sql_file;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use bincode::config;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn main() -> Result<()> {
    let mut page: HashMap<(i16, String), u32> = HashMap::new();
    parse_sql_file(
        "enwiki-latest-page.sql",
        Regex::new(r"\((\d+),(-?\d+),'((?:[^'\\]|\\.)*)',[\w\d.,']+\)").unwrap(),
        |cap| {
            let page_id = cap[1].parse::<u32>().unwrap();
            let namespace = cap[2].parse::<i16>().unwrap();
            let title = cap[3].to_string();

            page.insert((namespace, title), page_id);
        })?;

    let mut link_target: HashMap<u32, u32> = HashMap::new();
    parse_sql_file(
        "enwiki-latest-linktarget.sql",
        Regex::new(r"\((\d+),(-?\d+),'((?:[^'\\]|\\.)*)'\)").unwrap(),
        |cap| {
            let link_target_id = cap[1].parse::<u32>().unwrap();
            let namespace = cap[2].parse::<i16>().unwrap();
            let title = cap[3].to_string();

            if let Some(page_id) = page.get(&(namespace, title)) {
                link_target.insert(link_target_id, *page_id);
            }
        })?;

    page.clear();

    let mut links: HashMap<u32, u32> = HashMap::new();
    parse_sql_file(
        "enwiki-latest-pagelinks.sql",
        Regex::new(r"\((\d+),-?\d+,(\d+)\)").unwrap(),
        |cap| {
            let from_page_id = cap[1].parse::<u32>().unwrap();
            let to_link_target_id = cap[2].parse::<u32>().unwrap();

            if let Some(to_page_id) = link_target.get(&to_link_target_id) {
                links.insert(from_page_id, *to_page_id);
            }
        })?;

    // Using Bincode, encode links to a file
    let mut file = File::create("links.bin")?;
    bincode::encode_into_std_write(&links, &mut file, config::standard())?;

    Ok(())
}
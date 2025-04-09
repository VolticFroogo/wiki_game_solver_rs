use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;
use regex::Regex;
use crate::parse_sql_file::parse_sql_file;

// Parse the SQL files and extract a map of page ID to vector of page IDs it links to
// TODO: explain logic and DB structure
pub(crate) fn generate_and_save_link_mapping() -> crate::Result<()> {
    let mut page: HashMap<String, u32> = HashMap::new();
    parse_sql_file(
        "sql/enwiki-latest-page.sql",
        Regex::new(r"\((\d+),0,'((?:[^'\\]|\\.)*)',[\w\d.,']+\)").unwrap(),
        |cap| {
            let page_id = cap[1].parse::<u32>().unwrap();
            let title = cap[2].to_string();

            page.insert(title, page_id);
        })?;

    let mut link_target: HashMap<u32, u32> = HashMap::new();
    parse_sql_file(
        "sql/enwiki-latest-linktarget.sql",
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
    parse_sql_file(
        "sql/enwiki-latest-pagelinks.sql",
        Regex::new(r"\((\d+),0,(\d+)\)").unwrap(),
        |cap| {
            let from_page_id = cap[1].parse::<u32>().unwrap();
            let to_link_target_id = cap[2].parse::<u32>().unwrap();

            if let Some(to_page_id) = link_target.get(&to_link_target_id) {
                links.entry(from_page_id).or_insert_with(Vec::new).push(*to_page_id);
            }
        })?;

    write_to_bincode_file(&links)?;

    Ok(())
}

// Using Bincode, encode links to a file
fn write_to_bincode_file(links: &HashMap<u32, Vec<u32>>) -> crate::Result<()> {
    println!("Found {} pages with links, writing to links.bin", links.len());

    let start = Instant::now();
    let file = File::create("links/links.bin.tmp")?;
    let mut buffered_file = BufWriter::new(file);
    bincode::encode_into_std_write(&links, &mut buffered_file, bincode::config::standard())?;

    println!("Finished writing {} MiB to links.bin in {}s",
             buffered_file.get_ref().metadata()?.len() / (1024 * 1024),
             start.elapsed().as_secs());

    let moved_file = File::create("links/links.bin")?;
    fs2::FileExt::lock_exclusive(&moved_file)?;
    std::fs::rename("links/links.bin.tmp", "links/links.bin")?;
    fs2::FileExt::unlock(&moved_file)?;

    Ok(())
}

use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn main() -> Result<()> {
    let mut links_file = File::open("links.bin")?;
    println!("Reading links.bin ({} MiB) bincode", links_file.metadata()?.len() / (1024 * 1024));

    let mut buffered_links_file = BufReader::new(&mut links_file);

    let links: HashMap<u32, Vec<u32>> =
        bincode::decode_from_reader(&mut buffered_links_file, bincode::config::standard())?;

    println!("Read {} pages with links from links.bin", links.len());

    let mut link_queue: VecDeque<u32> = VecDeque::new();
    let mut seen_by: HashMap<u32, u32> = HashMap::new();
    let start = 62877952;
    let target = 29086544;

    link_queue.push_back(start);
    seen_by.insert(start, start);

    'outer: while let Some(current) = link_queue.pop_front() {
        if let Some(next_page) = links.get(&current) {
            for &next_link in next_page.iter() {
                if seen_by.contains_key(&next_link) {
                    continue;
                }

                seen_by.insert(next_link, current);

                if next_link == target {
                    break 'outer;
                }

                link_queue.push_back(next_link);
            }
        }
    }

    link_queue.clear();

    if !seen_by.contains_key(&target) {
        println!("No path found from {} to {}", start, target);
        return Ok(());
    }

    let mut path = Vec::new();
    let mut current = target;
    while let Some(&prev) = seen_by.get(&current) {
        path.push(current);
        current = prev;

        if current == start {
            break;
        }
    }
    path.push(start);

    for node in path.iter().rev() {
        print!("{}|", node);
    }
    println!();

    Ok(())
}

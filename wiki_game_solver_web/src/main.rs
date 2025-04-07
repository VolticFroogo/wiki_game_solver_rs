use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn main() -> Result<()> {
    let mut links_file = File::open("links.bin")?;
    println!("Reading links.bin ({} MiB) bincode", links_file.metadata()?.len() / (1024 * 1024));

    let mut buffered_links_file = BufReader::new(&mut links_file);

    let links: HashMap<u32, Vec<u32>> =
        bincode::decode_from_reader(&mut buffered_links_file, bincode::config::standard())?;

    println!("Read {} pages with links from links.bin", links.len());
    println!("Generating reverse link lookup");

    let mut links_reverse: HashMap<u32, Vec<u32>> = HashMap::new();
    for (from_page_id, to_page_ids) in links.iter() {
        for to_page_id in to_page_ids.iter() {
            links_reverse.entry(*to_page_id).or_insert_with(Vec::new).push(*from_page_id);
        }
    }

    println!("Generated reverse link lookup");

    let start_time = Instant::now();

    let mut forward_link_queue: Vec<u32> = Vec::new();
    let mut reverse_link_queue: Vec<u32> = Vec::new();
    let mut forward_seen_by: HashMap<u32, u32> = HashMap::new();
    let mut reverse_seen_by: HashMap<u32, u32> = HashMap::new();
    let start = 6912103;
    let target = 45567357;
    // let start = 62877952;
    // let target = 29086544;

    forward_link_queue.push(start);
    reverse_link_queue.push(target);
    forward_seen_by.insert(start, start);
    reverse_seen_by.insert(target, target);

    while !forward_link_queue.is_empty() && !reverse_link_queue.is_empty() {
        let mut new_forward_link_queue: Vec<u32> = Vec::new();

        if let Some(path) = bfs_directional_search_step(&links, &mut forward_link_queue, &mut forward_seen_by, &mut reverse_seen_by, &mut new_forward_link_queue) {
            println!("Found path in {}s", start_time.elapsed().as_secs_f64());
            print!("{}", path.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("|"));
            return Ok(());
        }

        forward_link_queue = new_forward_link_queue;
        let mut new_reverse_link_queue: Vec<u32> = Vec::new();

        if let Some(path) = bfs_directional_search_step(&links_reverse, &mut reverse_link_queue, &mut reverse_seen_by, &mut forward_seen_by, &mut new_reverse_link_queue) {
            println!("Found path in {}s", start_time.elapsed().as_secs_f64());
            print!("{}", path.iter().rev().map(|x| x.to_string()).collect::<Vec<String>>().join("|"));
            return Ok(());
        }

        reverse_link_queue = new_reverse_link_queue;
    }

    println!("Determined path unreachable in {}s", start_time.elapsed().as_secs_f64());
    Ok(())
}

fn bfs_directional_search_step(
    links: &HashMap<u32, Vec<u32>>,
    link_queue: &mut Vec<u32>,
    seen_by: &mut HashMap<u32, u32>,
    reverse_seen_by: &mut HashMap<u32, u32>,
    new_link_queue: &mut Vec<u32>)
    -> Option<Vec<u32>> {
    for &current_page_id in link_queue.iter() {
        if let Some(next_page_links) = links.get(&current_page_id) {
            for &next_page_id in next_page_links {
                if seen_by.contains_key(&next_page_id) {
                    continue;
                }

                seen_by.insert(next_page_id, current_page_id);

                if reverse_seen_by.contains_key(&next_page_id) {
                    return Some(bfs_generate_path(seen_by, reverse_seen_by, next_page_id));
                }

                new_link_queue.push(next_page_id);
            }
        }
    }

    None
}

fn bfs_generate_path(seen_by: &mut HashMap<u32, u32>, reverse_seen_by: &mut HashMap<u32, u32>, next_page_id: u32) -> Vec<u32> {
    let mut path = Vec::new();

    populate_path_with_seen_by_until_circular(seen_by, &mut path, next_page_id);

    path.reverse();
    path.push(next_page_id);

    populate_path_with_seen_by_until_circular(reverse_seen_by, &mut path, next_page_id);

    path
}

fn populate_path_with_seen_by_until_circular(seen_by: &mut HashMap<u32, u32>, path: &mut Vec<u32>, current_page_id: u32) {
    let mut current_page_id = current_page_id;

    while let Some(&next_page_id) = seen_by.get(&current_page_id) {
        if current_page_id == next_page_id {
            break;
        }

        path.push(next_page_id);
        current_page_id = next_page_id;
    }
}

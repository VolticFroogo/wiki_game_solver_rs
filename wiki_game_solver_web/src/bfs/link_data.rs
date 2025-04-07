use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

pub(crate) struct LinkData {
    pub links_forward: HashMap<u32, Vec<u32>>,
    pub links_reverse: HashMap<u32, Vec<u32>>,
}

pub(crate) fn get_link_data() -> crate::Result<LinkData> {
    let links_forward = read_links_from_file()?;
    let links_reverse = generate_links_reverse(&links_forward);

    Ok(LinkData {
        links_forward,
        links_reverse,
    })
}

fn read_links_from_file() -> crate::Result<HashMap<u32, Vec<u32>>> {
    let mut links_file = File::open("links.bin")?;
    println!("Reading links.bin ({} MiB) bincode", links_file.metadata()?.len() / (1024 * 1024));

    let mut buffered_links_file = BufReader::new(&mut links_file);

    let links: HashMap<u32, Vec<u32>> =
        bincode::decode_from_reader(&mut buffered_links_file, bincode::config::standard())?;

    println!("Read {} pages with links from links.bin", links.len());
    Ok(links)
}

fn generate_links_reverse(links: &HashMap<u32, Vec<u32>>) -> HashMap<u32, Vec<u32>> {
    println!("Generating reverse link lookup");

    let mut links_reverse: HashMap<u32, Vec<u32>> = HashMap::new();
    for (from_page_id, to_page_ids) in links.iter() {
        for to_page_id in to_page_ids.iter() {
            links_reverse.entry(*to_page_id).or_insert_with(Vec::new).push(*from_page_id);
        }
    }

    println!("Generated reverse link lookup");
    links_reverse
}

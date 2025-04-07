mod bfs;

use std::error::Error;
use std::time::Instant;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn main() -> Result<()> {
    let link_data = bfs::get_link_data()?;

    let start = 6912103;
    let target = 45567357;
    // let start = 62877952;
    // let target = 29086544;

    let start_time = Instant::now();
    let path = bfs::bfs_bidirectional(&link_data, start, target);
    if path.is_none() {
        println!("Path determined unreachable in {}s", start_time.elapsed().as_secs_f64());
        return Ok(());
    }

    println!("Found path in {}s", start_time.elapsed().as_secs_f64());
    println!("Path: {}", path.unwrap().iter().map(|x| x.to_string()).collect::<Vec<String>>().join("|"));

    Ok(())
}

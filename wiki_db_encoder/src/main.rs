mod parse_sql_file;
mod link_mapping;

use crate::link_mapping::generate_and_save_link_mapping;
use std::error::Error;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn main() -> Result<()> {
    generate_and_save_link_mapping()?;

    Ok(())
}

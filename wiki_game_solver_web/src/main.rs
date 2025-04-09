mod bfs;
mod endpoints;

use actix_web::web::Data;
use actix_web::{App, HttpServer};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[actix_web::main]
async fn main() -> Result<()> {
    let link_data = Arc::new(RwLock::new(bfs::get_link_data()?));

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(link_data.clone()))
            .service(endpoints::solve)
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await?;

    Ok(())
}

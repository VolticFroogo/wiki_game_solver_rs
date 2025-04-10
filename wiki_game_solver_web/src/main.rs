mod bfs;
mod endpoints;

use crate::bfs::LinkData;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[actix_web::main]
async fn main() -> Result<()> {
    let link_data: Arc<RwLock<Option<LinkData>>> = Arc::new(RwLock::new(None));

    let link_data_clone = link_data.clone();
    tokio::spawn(async move {
        let link_data_new = bfs::get_link_data()?;

        let mut link_data_acquired = link_data_clone.write().await;
        *link_data_acquired = Some(link_data_new);

        crate::Result::Ok(())
    });

    let link_data_clone = link_data.clone();
    tokio::spawn(async move {
        loop {
            let link_data_clone = link_data_clone.clone();
            if let Err(e) = bfs::watch_file(link_data_clone).await {
                eprintln!("Error in file watcher: {}", e);
                println!("Retrying in 1 hour...");
                tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await; // 1 hour
            }
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(link_data.clone()))
            .service(endpoints::solve)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}


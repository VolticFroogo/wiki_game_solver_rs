mod bfs;
mod endpoints;

use actix_web::web::Data;
use actix_web::{App, HttpServer};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::RwLock;
use crate::bfs::LinkData;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[actix_web::main]
async fn main() -> Result<()> {
    let link_data = bfs::get_link_data()?;
    let link_data = Arc::new(RwLock::new(link_data));

    let link_data_clone = link_data.clone();
    tokio::spawn(async move {
        if let Err(e) = watch_file(link_data_clone).await {
            eprintln!("Error in file watcher: {}", e);
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

async fn watch_file(link_data: Arc<RwLock<LinkData>>) -> Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    watcher.watch(Path::new("links/links.bin"), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                if !event.kind.is_modify() {
                    continue;
                }

                let link_data_new = bfs::get_link_data()?;
                let mut link_data_acquired = link_data.write().await;
                *link_data_acquired = link_data_new;
            },
            Err(e) => eprintln!("File watcher change error: {:?}", e),
        }
    }

    Ok(())
}

fn async_watcher() -> Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (tx, rx) = channel(1);
    let rt = Runtime::new()?;

    let watcher = RecommendedWatcher::new(
        move |res| {
            let tx = tx.clone();
            rt.spawn(async move {
                tx.send(res).await.unwrap();
            });
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

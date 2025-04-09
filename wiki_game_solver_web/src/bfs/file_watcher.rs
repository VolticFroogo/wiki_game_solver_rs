use std::path::Path;
use std::sync::Arc;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::RwLock;
use crate::bfs;
use crate::bfs::LinkData;

pub(crate) async fn watch_file(link_data: Arc<RwLock<LinkData>>) -> crate::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    watcher.watch(Path::new("links/links.bin"), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                if !event.kind.is_modify() {
                    continue;
                }

                println!("File watcher detected modification of links.bin, waiting 1m before reloading...");
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;

                let link_data_new = bfs::get_link_data()?;
                let mut link_data_acquired = link_data.write().await;
                *link_data_acquired = link_data_new;
            },
            Err(e) => eprintln!("File watcher change error: {:?}", e),
        }
    }

    Ok(())
}

fn async_watcher() -> crate::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
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

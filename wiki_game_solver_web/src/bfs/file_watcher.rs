use std::path::Path;
use std::sync::Arc;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::RwLock;
use tokio::time::Instant;
use crate::bfs;
use crate::bfs::LinkData;

pub(crate) async fn watch_file(link_data: Arc<RwLock<Option<LinkData>>>) -> crate::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    watcher.watch(Path::new("links/links.bin"), RecursiveMode::NonRecursive)?;
    let mut last_event_time: Option<Instant> = None;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                if !event.kind.is_modify() {
                    continue;
                }

                // Ignore events within 5 minutes of the last seen event
                if last_event_time.is_some_and(|last_seen|
                    last_seen.elapsed().as_secs() < 5 * 60) {
                    continue;
                }

                last_event_time = Some(Instant::now());

                update_link_data(link_data).await?;
            },
            Err(e) => eprintln!("File watcher change error: {:?}", e),
        }
    }

    Ok(())
}

async fn update_link_data(link_data: Arc<RwLock<Option<LinkData>>>) -> crate::Result<()> {
    println!("File watcher detected modification of links.bin, waiting 1m before reloading...");
    tokio::time::sleep(std::time::Duration::from_secs(60)).await;

    let link_data_new = bfs::get_link_data()?;
    let mut link_data_acquired = link_data.write().await;
    *link_data_acquired = Some(link_data_new);
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

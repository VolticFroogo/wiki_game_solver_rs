mod bfs;

use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use crate::bfs::LinkData;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Deserialize)]
struct Query {
    start: u32,
    target: u32,
}

#[derive(Serialize)]
struct Response {
    path: Vec<u32>,
    seconds_taken: f64,
}

#[get("/solve")]
async fn solve(link_data: web::Data<Arc<RwLock<LinkData>>>, query: web::Query<Query>) -> impl Responder {
    let link_data = link_data.read().await;

    let start_time = Instant::now();

    let path = bfs::bfs_bidirectional(link_data, query.start, query.target);
    if path.is_none() {
        println!("Path determined unreachable in {}s", start_time.elapsed().as_secs_f64());
        return HttpResponse::UnprocessableEntity().body("abc");
    }
    let path = path.unwrap();

    let seconds_taken = start_time.elapsed().as_secs_f64();
    println!("Found {} degree path from {} to {} in {}s", path.len()-1, query.start, query.target, seconds_taken);

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(serde_json::to_string(&Response{
            path,
            seconds_taken,
        }).unwrap())
}

#[actix_web::main]
async fn main() -> Result<()> {
    let link_data = Arc::new(RwLock::new(bfs::get_link_data()?));

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(link_data.clone()))
            .service(solve)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}

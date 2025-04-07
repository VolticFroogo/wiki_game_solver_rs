use std::sync::Arc;
use std::time::Instant;
use actix_web::{get, web, HttpResponse, Responder};
use actix_web::http::header::ContentType;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use crate::bfs;
use crate::bfs::LinkData;

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
pub async fn solve(link_data: web::Data<Arc<RwLock<LinkData>>>, query: web::Query<Query>) -> impl Responder {
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
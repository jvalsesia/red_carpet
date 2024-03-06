use std::fs::File;

use log::info;
use tokio::net::TcpListener;

use crate::{persistence::create_persistence_file, routes::define_routes};

pub mod errors;
pub mod handlers;
pub mod models;
pub mod persistence;
pub mod routes;
pub mod utils;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Avaya Rust Red Carpet");

    let _ = create_persistence_file();

    let app = define_routes();

    // `axum::Server` is a re-export of `hyper::Server`
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!("listening on: {:?}", listener);

    axum::serve(listener, app.await.into_make_service())
        .await
        .unwrap();
}

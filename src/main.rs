use database::employee_db;
use log::info;
use tera::Tera;
use tokio::net::TcpListener;

use crate::{persistence::create_persistence_store, routes::define_routes};

pub mod api;
pub mod database;
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

    let _ = create_persistence_store();

    let db = employee_db();

    let tera = Tera::default();

    let app = define_routes(db, tera);

    // `axum::Server` is a re-export of `hyper::Server`
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!("listening on: {:?}", listener);

    axum::serve(listener, app.await.into_make_service())
        .await
        .unwrap();
}

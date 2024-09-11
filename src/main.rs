use std::{collections::HashMap, sync::Arc};

use database::{
    file_manager::FileManager,
    persistence::{create_admin, create_persistence_store, ADMIN_DATA_FILE, EMPLOYEE_DATA_FILE},
};
use log::info;
use models::admin_models::Admin;
use routes::define_routes;
use tera::Tera;
use tokio::{net::TcpListener, sync::Mutex};
use utils::state::AppState;

pub mod database;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod utils;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Avaya Rust Red Carpet");

    let _ = create_persistence_store();

    let admin = Admin {
        id: "admin".to_string(),
        password: Some("admin".to_string()),
    };

    let admin_created = create_admin(admin).await;
    if admin_created.unwrap_or(true) {
        info!("Admin created successfully");
    } else {
        info!("Admin already exists");
    }

    let tera = Tera::default();

    let file_manager = Arc::new(FileManager::new(EMPLOYEE_DATA_FILE, ADMIN_DATA_FILE).unwrap());

    let state = AppState {
        sessions: Arc::new(Mutex::new(HashMap::new())),
        file_manager,
    };

    let app = define_routes(state, tera);

    // `axum::Server` is a re-export of `hyper::Server`
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!("listening on: {:?}", listener);

    axum::serve(listener, app.await.into_make_service())
        .await
        .unwrap();
}

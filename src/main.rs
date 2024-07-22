use std::sync::Arc;

use backend::routes::define_routes;

use database::persistence::create_persistence_store;
use log::info;
use tera::Tera;
use tokio::{net::TcpListener, sync::Mutex};
use utils::state::LoggedInState;

pub mod backend;
pub mod database;
pub mod models;
pub mod frontend;
pub mod utils;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Avaya Rust Red Carpet");

    let _ = create_persistence_store();

    //let db = employee_db();

    // Initialize the application state with no logged-in user
    let logged_in_state = Arc::new(Mutex::new(LoggedInState { user: None }));

    let tera = Tera::default();

    let app = define_routes(logged_in_state, tera);

    // `axum::Server` is a re-export of `hyper::Server`
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!("listening on: {:?}", listener);

    axum::serve(listener, app.await.into_make_service())
        .await
        .unwrap();
}

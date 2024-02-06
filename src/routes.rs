use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use tera::Tera;

use crate::{
    handlers::{
        create_employee, employees_list, generate_handle_and_password, get_employee,
        health_checker, index,
    },
    models,
};

pub async fn define_routes() -> Router {
    let db = models::employee_db();

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("base.html", include_str!("./templates/base.html")),
        ("welcome.html", include_str!("./templates/welcome.html")),
        ("index", include_str!("./templates/index.html")),
    ])
    .unwrap();
    // build our application with a route
    Router::new()
        .route("/api/v1/healthchecker", get(health_checker))
        .route(
            "/api/v1/employees",
            post(create_employee).get(employees_list),
        )
        .route(
            "/api/v1/employees/:id",
            get(get_employee).patch(generate_handle_and_password),
        )
        .route("/", get(index))
        .layer(Extension(Arc::new(tera)))
        .with_state(db)
}

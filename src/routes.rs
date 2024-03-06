use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use tera::Tera;

use crate::{
    handlers::{
        create_employee, employees_list, generate_handle_and_password, get_employee,
        health_checker, index, styles,
    },
    models,
};

pub async fn define_routes() -> Router {
    let db = models::employee_db();

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("index", include_str!("./templates/index.html")),
        ("base.html", include_str!("./templates/base.html")),
        ("employees", include_str!("./templates/employees.html")),
        ("employee", include_str!("./templates/employee.html")),
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
        .route("/styles.css", get(styles))
        .route("/employees", get(employees_list))
        .route("/employee/:id", get(get_employee))
        .layer(Extension(Arc::new(tera)))
        .with_state(db)
}

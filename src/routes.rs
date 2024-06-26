use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use tera::Tera;

use crate::{
    database::DB,
    handlers::{
        create_employee, delete_employee, edit_employee, employees_list,
        generate_handle_and_password, get_employee, health_checker, index, list_employees,
        select_employee,
    },
};

pub async fn define_routes(db: DB, mut tera: Tera) -> Router {
    tera.add_raw_templates(vec![
        ("base.html", include_str!("./templates/base.html")),
        ("index.html", include_str!("./templates/index.html")),
        ("employees.html", include_str!("./templates/employees.html")),
        ("edit_form.html", include_str!("./templates/edit_form.html")),
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
        .route("/employees", get(list_employees))
        .route("/employee/:id", get(edit_employee))
        .route("/delete/employee/:id", get(delete_employee))
        .route("/select/employee/:id", get(select_employee))
        .layer(Extension(Arc::new(tera)))
        .with_state(db)
}

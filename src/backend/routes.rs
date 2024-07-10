use std::sync::Arc;

use axum::{
    routing::{any, get, post},
    Extension, Router,
};
use tera::Tera;

use crate::{
    database::database::DB,
    frontend::handlers::{
        delete_employee, edit_employee, handle_edit_form_data, handle_onboard_form_data,
        handle_save_form_data, index, list_employees, new_employee_page, save_result_page,
        secure_password, select_employee, styles,
    },
};

use super::api::{
    create_employee, employees_list, generate_handle_and_password, get_employee, health_checker,
};

pub async fn define_routes(db: DB, mut tera: Tera) -> Router {
    tera.add_raw_templates(vec![
        ("base.html", include_str!("../frontend/templates/base.html")),
        (
            "index.html",
            include_str!("../frontend/templates/index.html"),
        ),
        (
            "employee.html",
            include_str!("../frontend/templates/employee.html"),
        ),
        (
            "employees.html",
            include_str!("../frontend/templates/employees.html"),
        ),
        (
            "new_employee.html",
            include_str!("../frontend/templates/new_employee.html"),
        ),
        (
            "save_result.html",
            include_str!("../frontend/templates/save_result.html"),
        ),
        (
            "edit_form.html",
            include_str!("../frontend/templates/edit_form.html"),
        ),
        (
            "delete_confirmation.html",
            include_str!("../frontend/templates/delete_confirmation.html"),
        ),
        (
            "errors.html",
            include_str!("../frontend/templates/errors.html"),
        ),
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
        .route("/styles.css", any(styles))
        .route("/", get(index))
        .route("/list/employees", get(list_employees))
        .route("/edit/employee/:id", get(edit_employee))
        .route("/update/employee", post(handle_edit_form_data))
        .route("/onboard/employee", post(handle_onboard_form_data))
        .route("/securepassword/employee", post(secure_password))
        .route("/new/employee", get(new_employee_page))
        .route("/save/employee", post(handle_save_form_data))
        .route("/save/success", get(save_result_page))
        .route("/delete/employee/:id", get(delete_employee))
        .route("/select/employee/:id", get(select_employee))
        .layer(Extension(Arc::new(tera)))
        .with_state(db)
}

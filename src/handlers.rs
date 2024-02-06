use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension, Json,
};
use log::debug;
use tera::{Context, Tera};
use uuid::Uuid;

use crate::models::{
    Employee, EmployeeData, EmployeeListResponse, QueryOptions, SimpleEmployeeResponse, DB,
};

type Templates = Arc<Tera>;

pub async fn health_checker() -> impl IntoResponse {
    const MESSAGE: &str = "Avaya Rust Red Carpet";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn create_employee(
    State(db): State<DB>,
    Json(mut body): Json<Employee>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut vec = db.lock().await;

    if let Some(employee) = vec.iter().find(|employee: &&Employee| {
        employee.first_name == body.first_name && employee.last_name == body.last_name
    }) {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Employee: '{} {}' already exists!", employee.first_name, employee.last_name),
        });
        return Err((StatusCode::CONFLICT, Json(error_response)));
    }

    if body.age < 18 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Employee: '{} {}' is not 18 years old!", body.first_name, body.last_name),
        });
        return Err((StatusCode::EXPECTATION_FAILED, Json(error_response)));
    }

    if body.diploma.is_empty() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Employee: '{} {}' does not have diploma!", body.first_name, body.last_name),
        });
        return Err((StatusCode::EXPECTATION_FAILED, Json(error_response)));
    }

    let uuid_id = Uuid::new_v4();
    //let datetime = chrono::Utc::now();

    body.id = Some(uuid_id);
    body.onboarded = Some(false);

    let employee = body.to_owned();

    vec.push(body);

    let json_response = SimpleEmployeeResponse {
        status: "success".to_string(),
        data: EmployeeData { employee },
    };
    debug!("{json_response:?}");
    Ok((StatusCode::CREATED, Json(json_response)))
}

pub async fn employees_list(
    opts: Option<Query<QueryOptions>>,
    State(db): State<DB>,
) -> impl IntoResponse {
    let employees = db.lock().await;

    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    // list not onboarded employees
    let employees: Vec<Employee> = employees
        .clone()
        .into_iter()
        .skip(offset)
        .take(limit)
        .filter(|employee: &Employee| employee.onboarded == Some(false))
        .collect();

    let json_response = EmployeeListResponse {
        status: "success".to_string(),
        results: employees.len(),
        todos: employees,
    };

    debug!("{json_response:?}");
    Json(json_response)
}
pub async fn index(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    Html(templates.render("index", &Context::new()).unwrap())
}

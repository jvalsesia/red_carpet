use std::collections::HashMap;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use log::debug;

use uuid::Uuid;

use crate::{
    database::persistence::{delete, get_employee_by_handle, list, save, update},
    models::employee_models::{
        Employee, EmployeeErrorResponse, EmployeeListResponse, EmployeeRequestBody,
        EmployeeResponse, QueryOptions,
    },
    utils::password_utils::{generate_handle, generate_random_password},
};

pub async fn health_checker() -> impl IntoResponse {
    const MESSAGE: &str = "Avaya Rust Red Carpet";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn delete_employee_by_id(
    Path(id): Path<String>,
) -> Result<Json<EmployeeListResponse>, (StatusCode, Json<EmployeeErrorResponse>)> {
    let employees_map = delete(id.clone()).await;
    match employees_map {
        Ok(employees) => {
            let json_response = EmployeeListResponse {
                message: format!("Employee {id:?} deleted successfully"),
                results: employees.len(),
                employees: employees.values().cloned().collect(),
            };
            debug!("{json_response:?}");
            Ok(Json(json_response))
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                error: error.to_string(),
            };
            Err((StatusCode::NOT_MODIFIED, Json(error_response)))
        }
    }
}

pub async fn update_employee(
    Json(body): Json<Employee>,
) -> Result<Json<EmployeeListResponse>, (StatusCode, Json<EmployeeErrorResponse>)> {
    let employee = body.clone();
    let employees_map = update(employee).await;
    match employees_map {
        Ok(employees) => {
            let id = body.id.clone().unwrap();
            let mut vec_employees: Vec<Employee> = employees.values().cloned().collect();
            vec_employees.sort_by(|x, y| x.first_name.cmp(&y.first_name));

            let json_response = EmployeeListResponse {
                message: format!("Employee {id:?} updated successfully"),
                results: employees.len(),
                employees: vec_employees,
            };

            debug!("{json_response:?}");
            Ok(Json(json_response))
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                error: error.to_string(),
            };
            Err((StatusCode::NOT_MODIFIED, Json(error_response)))
        }
    }
}

pub async fn create_employee(
    Json(body): Json<EmployeeRequestBody>,
) -> Result<Json<EmployeeResponse>, (StatusCode, Json<EmployeeErrorResponse>)> {
    let employee = Employee {
        id: Some(Uuid::new_v4().to_string()),
        first_name: body.first_name,
        last_name: body.last_name,
        personal_email: body.personal_email,
        avaya_email: None,
        age: body.age,
        diploma: body.diploma,
        onboarded: Some(false),
        handle: None,
        password: None,
        secure_password: Some(false),
    };
    let save_result = save(employee.clone()).await;
    match save_result {
        Ok(_) => {
            let json_response = EmployeeResponse {
                message: "Employee created successfully".to_string(),
                data: employee,
            };
            debug!("{json_response:?}");
            Ok(Json(json_response))
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                error: error.to_string(),
            };
            Err((StatusCode::NOT_MODIFIED, Json(error_response)))
        }
    }
}

pub async fn employees_list(
    opts: Option<Query<QueryOptions>>,
) -> Result<Json<EmployeeListResponse>, (StatusCode, Json<EmployeeErrorResponse>)> {
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let employees_list = list().await;

    match employees_list {
        Ok(employees) => {
            let filtered_employees: HashMap<String, Employee> = employees
                .into_iter()
                .skip(offset)
                .take(limit)
                // .filter(|(_id, employee)| employee.onboarded == Some(false))
                .collect();

            let mut vec_employees: Vec<Employee> = filtered_employees.values().cloned().collect();
            vec_employees.sort_by(|x, y| x.first_name.cmp(&y.first_name));

            let json_response = EmployeeListResponse {
                message: "Employees list".to_string(),
                results: filtered_employees.len(),
                employees: vec_employees,
            };
            debug!("{json_response:?}");
            Ok(Json(json_response))
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                error: error.to_string(),
            };
            Err((StatusCode::NOT_MODIFIED, Json(error_response)))
        }
    }
}

pub async fn get_employee(
    Path(id): Path<String>,
) -> Result<Json<EmployeeResponse>, (StatusCode, Json<EmployeeErrorResponse>)> {
    let employees_list = list().await;

    match employees_list {
        Ok(employees) => {
            // list not onboarded employees
            let filtered_employees: HashMap<String, Employee> = employees
                .into_iter()
                .filter(|(_id, employee)| employee.id == Some(id.clone()))
                .collect();

            let json_response = EmployeeResponse {
                message: "Employee found".to_string(),
                data: filtered_employees.get(&id).unwrap().clone(),
            };
            debug!("{json_response:?}");
            Ok(Json(json_response))
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                error: error.to_string(),
            };
            Err((StatusCode::NOT_MODIFIED, Json(error_response)))
        }
    }
}

pub async fn get_employee_by_handle_2(
    Path(handle): Path<String>,
) -> Result<Json<EmployeeResponse>, (StatusCode, Json<EmployeeErrorResponse>)> {
    let employee_by_handle = get_employee_by_handle(handle.clone()).await;

    match employee_by_handle {
        Ok(employee) => {
            let json_response = EmployeeResponse {
                message: "Employee found".to_string(),
                data: employee,
            };
            debug!("{json_response:?}");
            Ok(Json(json_response))
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                error: error.to_string(),
            };
            Err((StatusCode::NOT_MODIFIED, Json(error_response)))
        }
    }
}
pub async fn generate_handle_and_password(Path(id): Path<String>) -> impl IntoResponse {
    let employees_list = list().await;

    match employees_list {
        Ok(employees) => {
            // list not onboarded employees

            let filtered_employee = employees.get(&id).unwrap();

            let new_handle = generate_handle(
                filtered_employee.first_name.clone(),
                filtered_employee.last_name.clone(),
            )
            .await;

            let employee = Employee {
                id: filtered_employee.id.clone(),
                first_name: filtered_employee.first_name.clone(),
                last_name: filtered_employee.last_name.clone(),
                personal_email: filtered_employee.personal_email.clone(),
                avaya_email: Some(format!("{}@avaya.com", new_handle)),
                age: filtered_employee.age,
                diploma: filtered_employee.diploma.clone(),
                onboarded: Some(true),
                handle: Some(new_handle),
                password: Some(generate_random_password().await),
                secure_password: Some(false),
            };

            let save_result = save(employee.clone()).await;
            match save_result {
                Ok(_) => {
                    let json_response = EmployeeResponse {
                        message: "Employee onboarded successfully".to_string(),
                        data: employee,
                    };
                    debug!("{json_response:?}");
                    Ok((StatusCode::OK, Json(json_response)))
                }
                Err(error) => {
                    debug!("{error:?}");
                    let json_response = EmployeeErrorResponse {
                        error: error.to_string(),
                    };
                    Err((StatusCode::NOT_MODIFIED, Json(json_response)))
                }
            }
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                error: error.to_string(),
            };
            Err((StatusCode::NOT_MODIFIED, Json(error_response)))
        }
    }
}

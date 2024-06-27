use std::{collections::HashMap, f32::consts::E, fs, path, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::{self, Response, StatusCode},
    response::{ErrorResponse, Html, IntoResponse},
    Extension, Form, Json,
};
use log::{debug, info};
use tera::{Context, Tera};
use uuid::Uuid;

use crate::{
    database::DB,
    errors::{
        employee_already_exists_error, employee_no_diploma_error, employee_not_old_enough_error,
    },
    models::{
        Employee, EmployeeData, EmployeeErrorResponse, EmployeeListResponse, EmployeeRequestBody,
        QueryOptions, SimpleEmployeeResponse,
    },
    persistence::{delete, list, save, update},
    utils::{generate_handle, generate_random_password},
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
    Json(mut body): Json<EmployeeRequestBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let employee = Employee {
        id: Some(Uuid::new_v4().to_string()),
        first_name: body.first_name,
        last_name: body.last_name,
        email: None,
        age: body.age,
        diploma: body.diploma,
        onboarded: Some(false),
        handle: None,
        password: None,
    };

    let save_result = save(employee.clone()).await;
    match save_result {
        Ok(_) => {
            let json_response = SimpleEmployeeResponse {
                status: "success".to_string(),
                data: EmployeeData { employee },
            };
            Ok((StatusCode::CREATED, Json(json_response)))
        }
        Err(error) => {
            debug!("{error:?}");
            let json_response = SimpleEmployeeResponse {
                status: "error".to_string(),
                data: EmployeeData { employee },
            };
            Ok((StatusCode::NOT_MODIFIED, Json(json_response)))
        }
    }
}

pub async fn employees_list(
    opts: Option<Query<QueryOptions>>,
    Extension(templates): Extension<Templates>,
    State(db): State<DB>,
) -> impl IntoResponse {
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
                .filter(|(_id, employee)| employee.onboarded == Some(false))
                .collect();

            let json_response = EmployeeListResponse {
                status: "success".to_string(),
                results: filtered_employees.len(),
                employees: filtered_employees.clone(),
            };
            debug!("{json_response:?}");
            Ok((StatusCode::OK, Json(json_response)))
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                status: "error".to_string(),
                description: error.to_string(),
            };
            Err((StatusCode::NOT_MODIFIED, Json(error_response)))
        }
    }
}

pub async fn get_employee(
    Path(id): Path<String>,
    Extension(templates): Extension<Templates>,
    State(db): State<DB>,
) -> impl IntoResponse {
    //let vec = db.lock().await;

    let employees_list = list().await;

    match employees_list {
        Ok(employees) => {
            // list not onboarded employees
            let filtered_employees: HashMap<String, Employee> = employees
                .into_iter()
                .filter(|(_id, employee)| employee.id == Some(id.clone()))
                .collect();

            let json_response = EmployeeListResponse {
                status: "success".to_string(),
                results: filtered_employees.len(),
                employees: filtered_employees.clone(),
            };
            debug!("{json_response:?}");
            Ok((StatusCode::OK, Json(json_response)))
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                status: "error".to_string(),
                description: error.to_string(),
            };
            Err((StatusCode::NOT_MODIFIED, Json(error_response)))
        }
    }
}

pub async fn generate_handle_and_password(
    Path(id): Path<String>,
    State(db): State<DB>,
) -> impl IntoResponse {
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
                email: Some(format!("{}@avaya.com", new_handle)),
                age: filtered_employee.age,
                diploma: filtered_employee.diploma.clone(),
                onboarded: Some(true),
                handle: Some(new_handle),
                password: Some(generate_random_password().await),
            };

            let save_result = save(employee.clone()).await;
            match save_result {
                Ok(_) => {
                    let json_response = SimpleEmployeeResponse {
                        status: "success".to_string(),
                        data: EmployeeData {
                            employee: employee.clone(),
                        },
                    };
                    debug!("{json_response:?}");
                    Ok((StatusCode::OK, Json(json_response)))
                }
                Err(error) => {
                    debug!("{error:?}");
                    let json_response = SimpleEmployeeResponse {
                        status: "error".to_string(),
                        data: EmployeeData { employee },
                    };
                    Ok((StatusCode::NOT_MODIFIED, Json(json_response)))
                }
            }
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                status: "error".to_string(),
                description: error.to_string(),
            };
            Err((StatusCode::NOT_MODIFIED, Json(error_response)))
        }
    }
}

pub async fn index(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Welcome to Avaya Red Carpet");

    Html(templates.render("index.html", &context).unwrap())
}

pub async fn list_employees(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "List Employees");
    context.insert("selected_id", "");
    let employees_map = list().await;
    match employees_map {
        Ok(employees) => {
            let mut employees_list: Vec<Employee> = employees.into_values().collect();
            employees_list.sort_by(|x, y| x.first_name.cmp(&y.first_name));
            debug!("{employees_list:?}");

            context.insert("employees", &employees_list);

            Html(templates.render("employees.html", &context).unwrap())
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                status: "error".to_string(),
                description: error.to_string(),
            };

            context.insert("error", &error_response);
            Html(templates.render("index.html", &context).unwrap())
        }
    }
}

pub async fn edit_employee(
    Path(id): Path<String>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Edit Employee");

    let employees_list = list().await;

    match employees_list {
        Ok(employees) => {
            // list not onboarded employees
            let filtered_employees: HashMap<String, Employee> = employees
                .into_iter()
                .filter(|(_id, employee)| employee.id == Some(id.clone()))
                .collect();

            let employees_list: Vec<Employee> = filtered_employees.into_values().collect();

            let employee = employees_list.first().unwrap();
            debug!("{employee:?}");

            context.insert("employee", &employee);

            Html(templates.render("edit_form.html", &context).unwrap())
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                status: "error".to_string(),
                description: error.to_string(),
            };

            context.insert("error", &error_response);
            Html(templates.render("index.html", &context).unwrap())
        }
    }
}

pub async fn delete_employee(
    Path(id): Path<String>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Edit Employee");

    let employees_list = delete(id).await;

    match employees_list {
        Ok(employees) => {
            let mut employees_list: Vec<Employee> = employees.into_values().collect();
            employees_list.sort_by(|x, y| x.first_name.cmp(&y.first_name));
            context.insert("employees", &employees_list);

            Html(templates.render("employees.html", &context).unwrap())
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                status: "error".to_string(),
                description: error.to_string(),
            };

            context.insert("error", &error_response);
            Html(templates.render("index.html", &context).unwrap())
        }
    }
}

pub async fn select_employee(
    Path(id): Path<String>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Selected Employee");
    context.insert("is_self", &true);
    let employees_list = list().await;

    match employees_list {
        Ok(employees) => {
            // list not onboarded employees
            let filtered_employees: HashMap<String, Employee> = employees
                .into_iter()
                .filter(|(_id, employee)| employee.id == Some(id.clone()))
                .collect();

            let employees_list: Vec<Employee> = filtered_employees.into_values().collect();

            let employee = employees_list.first().unwrap();

            context.insert("employee", &employee);

            Html(templates.render("employee.html", &context).unwrap())
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                status: "error".to_string(),
                description: error.to_string(),
            };

            context.insert("error", &error_response);
            Html(templates.render("index.html", &context).unwrap())
        }
    }
}

pub async fn handle_edit_form_data(
    Extension(templates): Extension<Templates>,
    Form(modified_employee_data): Form<Employee>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Edit Employee");
    debug!(
        "modified_employee_data.id ---> {:?}",
        modified_employee_data.id
    );
    let modified_employee = Employee {
        id: modified_employee_data.id,
        first_name: modified_employee_data.first_name.clone(),
        last_name: modified_employee_data.last_name.clone(),
        email: modified_employee_data.email.clone(),
        age: modified_employee_data.age,
        diploma: modified_employee_data.diploma.clone(),
        onboarded: modified_employee_data.onboarded,
        handle: modified_employee_data.handle.clone(),
        password: modified_employee_data.password.clone(),
    };

    debug!("modified_employee ---> {modified_employee:?}");
    let employees_map = update(modified_employee).await;

    match employees_map {
        Ok(employees) => {
            let mut employees_list: Vec<Employee> = employees.into_values().collect();
            employees_list.sort_by(|x, y| x.first_name.cmp(&y.first_name));
            debug!("{employees_list:?}");

            context.insert("employees", &employees_list);

            Html(templates.render("employees.html", &context).unwrap())
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                status: "error".to_string(),
                description: error.to_string(),
            };

            context.insert("error", &error_response);
            Html(templates.render("index.html", &context).unwrap())
        }
    }
}

pub async fn styles() -> impl IntoResponse {
    Response::builder()
        .status(http::StatusCode::OK)
        .header("Content-Type", "text/css")
        .body(include_str!("./public/styles.css").to_owned())
        .unwrap()
}

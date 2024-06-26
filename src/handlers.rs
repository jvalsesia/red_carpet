use std::{collections::HashMap, fs, path, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::{self, Response, StatusCode},
    response::{ErrorResponse, Html, IntoResponse},
    Extension, Json,
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
    persistence::{delete, list, save},
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
    // let mut vec = db.lock().await;

    // if let Some(_employee) = vec.iter().find(|employee: &&Employee| {
    //     employee.first_name == body.first_name && employee.last_name == body.last_name
    // }) {
    //     let error_response = serde_json::json!({
    //         "status": "fail",
    //         "message": employee_already_exists_error(body.first_name, body.last_name).err().unwrap().to_string(),
    //     });
    //     return Err((StatusCode::CONFLICT, Json(error_response)));
    // }

    // if body.age < 18 {
    //     let error_response = serde_json::json!({
    //         "status": "fail",
    //         "message": employee_not_old_enough_error(body.first_name, body.last_name).err().unwrap().to_string(),
    //     });
    //     return Err((StatusCode::EXPECTATION_FAILED, Json(error_response)));
    // }

    // if body.diploma.is_empty() {
    //     let error_response = serde_json::json!({
    //         "status": "fail",
    //         "message": employee_no_diploma_error( body.first_name, body.last_name).err().unwrap().to_string(),
    //     });
    //     return Err((StatusCode::EXPECTATION_FAILED, Json(error_response)));
    // }

    let employee = Employee {
        id: Some(Uuid::new_v4()),
        first_name: body.first_name,
        last_name: body.last_name,
        email: None,
        age: body.age,
        diploma: body.diploma,
        onboarded: Some(false),
        handle: None,
        password: None,
    };

    //vec.push(body);

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
    //let employees = db.lock().await;

    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    // let employee_file_path = path::Path::new("data/employee.json");

    // let data = fs::read_to_string(employee_file_path).expect("Unable to read file");

    // let mut employees: Vec<Employee> = Vec::new();
    // if fs::metadata(employee_file_path).unwrap().len() != 0 {
    //     employees = serde_json::from_str(&data).unwrap();
    // }
    //debug!("{employees:?}");

    // Ok(employees)

    let employees_list = list().await;

    match employees_list {
        Ok(employees) => {
            let filtered_employees: HashMap<Uuid, Employee> = employees
                .into_iter()
                .skip(offset)
                .take(limit)
                .filter(|(_id, employee)| employee.onboarded == Some(false))
                .collect();

            // // list not onboarded employees
            // let filtered_employees: Vec<Employee> = employees
            //     .into_iter()
            //     .skip(offset)
            //     .take(limit)
            //     .filter(|employee| employee.onboarded == Some(false))
            //     .collect();
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

    //let filtered_employees = employees;

    //debug!("{filtered_employees:?}");

    //let mut context = Context::new();
    //context.insert("employees", &employees.to_owned());

    //Html(templates.render("employees", &context).unwrap())
}

pub async fn get_employee(
    Path(id): Path<Uuid>,
    Extension(templates): Extension<Templates>,
    State(db): State<DB>,
) -> impl IntoResponse {
    //let vec = db.lock().await;

    let employees_list = list().await;

    match employees_list {
        Ok(employees) => {
            // list not onboarded employees
            let filtered_employees: HashMap<Uuid, Employee> = employees
                .into_iter()
                .filter(|(_id, employee)| employee.id == Some(id))
                .collect();
            // let filtered_employees: Vec<Employee> = employees
            //     .into_iter()
            //     .filter(|employee| employee.id == Some(id))
            //     .collect();
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
    // if let Some(employee) = vec
    //     .iter()
    //     .find(|employee: &&Employee| employee.id == Some(id))
    // {
    //     let json_response = SimpleEmployeeResponse {
    //         status: "success".to_string(),
    //         data: EmployeeData {
    //             employee: employee.clone(),
    //         },
    //     };
    //     debug!("{json_response:?}");
    //     let mut context = Context::new();
    //     context.insert("employee", &employee);

    //     //return Ok((StatusCode::OK, Json(json_response)));
    //     return Ok(Html(templates.render("employee", &context).unwrap()));
    // }

    // let error_response = serde_json::json!({
    //     "status": "fail",
    //     "message": format!("Employee with ID: {} not found", id)
    // });
    // debug!("{error_response:?}");
    // Err((StatusCode::NOT_FOUND, Json(error_response)))
}

pub async fn generate_handle_and_password(
    Path(id): Path<Uuid>,
    State(db): State<DB>,
) -> impl IntoResponse {
    let employees_list = list().await;

    match employees_list {
        Ok(employees) => {
            // list not onboarded employees

            // let filtered_employees: HashMap<Uuid, Employee> = employees
            //     .into_iter()
            //     .filter(|(_id, employee)| employee.id == Some(id))
            //     .collect();
            let filtered_employee = employees.get(&id).unwrap();

            // let filtered_employee = employees
            //     .into_iter()
            //     .find(|employee| employee.id == Some(id))
            //     .unwrap();

            let new_handle = generate_handle(
                filtered_employee.first_name.clone(),
                filtered_employee.last_name.clone(),
            )
            .await;

            let employee = Employee {
                id: filtered_employee.id,
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

    // let mut vec = db.lock().await;

    // if let Some(employee) = vec.iter_mut().find(|employee| employee.id == Some(id)) {
    //     let new_handle =
    //         generate_handle(employee.first_name.clone(), employee.last_name.clone()).await;

    //     let payload = Employee {
    //         id: employee.id,
    //         first_name: employee.first_name.clone(),
    //         last_name: employee.last_name.clone(),
    //         email: Some(format!("{}@avaya.com", new_handle)),
    //         age: employee.age,
    //         diploma: employee.diploma.clone(),
    //         onboarded: Some(true),
    //         handle: Some(new_handle),
    //         password: Some(generate_random_password().await),
    //     };
    //     *employee = payload;

    //     let json_response = SimpleEmployeeResponse {
    //         status: "success".to_string(),
    //         data: EmployeeData {
    //             employee: employee.clone(),
    //         },
    //     };
    //     Ok((StatusCode::OK, Json(json_response)))
    // } else {
    //     let error_response = serde_json::json!({
    //         "status": "fail",
    //         "message": format!("Employee with ID: {} not found", id)
    //     });

    //     Err((StatusCode::NOT_FOUND, Json(error_response)))
    // }
}

pub async fn index(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Welcome to Avaya Red Carpet");

    Html(templates.render("index.html", &context).unwrap())
}

pub async fn list_employees(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "List Employees");

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
    Path(id): Path<Uuid>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Edit Employee");

    let employees_list = list().await;

    match employees_list {
        Ok(employees) => {
            // list not onboarded employees
            let filtered_employees: HashMap<Uuid, Employee> = employees
                .into_iter()
                .filter(|(_id, employee)| employee.id == Some(id))
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
    Path(id): Path<Uuid>,
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
pub async fn styles() -> impl IntoResponse {
    Response::builder()
        .status(http::StatusCode::OK)
        .header("Content-Type", "text/css")
        .body(include_str!("./public/styles.css").to_owned())
        .unwrap()
}

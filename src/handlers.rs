use std::sync::Arc;

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use axum_auth::AuthBasic;
use log::debug;

use uuid::Uuid;

use crate::{
    database::persistence::{
        delete, get_admin_by_id, get_employee_by_handle, get_employee_by_id, list, save, update,
    },
    models::{
        admin_models::Admin,
        employee_models::{
            Employee, EmployeeErrorResponse, EmployeeListResponse, EmployeeRequestBody,
            EmployeeResponse, QueryOptions,
        },
    },
    utils::{
        password_utils::{
            self, generate_handle, generate_random_password, generate_session_token, hash_password,
            validate_token_expiration,
        },
        state::AppState,
    },
};
use axum::{
    http::{self, Response},
    response::Html,
    Form,
};
use log::{error, warn};

use tera::{Context, Tera};

type Templates = Arc<Tera>;

pub async fn styles() -> impl IntoResponse {
    Response::builder()
        .status(http::StatusCode::OK)
        .header("Content-Type", "text/css")
        .body(include_str!("./frontend/public/styles.css").to_owned())
        .unwrap()
}

pub async fn index(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Welcome to Avaya Red Carpet");

    Html(templates.render("index.html", &context).unwrap())
}

pub async fn login(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Login to Avaya Red Carpet");

    Html(templates.render("login.html", &context).unwrap())
}

pub async fn already_logged_ind(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Admin already logged in");

    Html(
        templates
            .render("already_logged_in.html", &context)
            .unwrap(),
    )
}

pub async fn errors(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Error");

    Html(templates.render("errors.html", &context).unwrap())
}

pub async fn new_employee_page(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Personal Details");

    Html(templates.render("new_employee.html", &context).unwrap())
}

pub async fn save_result_page(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Personal Details");

    Html(templates.render("save_result.html", &context).unwrap())
}

pub async fn list_employees(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Admin Dashboard");
    list_employees_renderer(context, templates).await
}

async fn validate_session_token(state: AppState, handle: String) -> bool {
    let sessions = state.sessions.lock().await;
    let token = sessions.get(&handle);
    warn!("validate token ---> {:?}", token);
    if token.is_none() {
        return false;
    }
    validate_token_expiration(token.unwrap().to_string()).await
}

async fn verify_handle_password(
    handle: String,
    password: String,
) -> Result<bool, (StatusCode, &'static str)> {
    let employee = get_employee_by_handle(handle.clone()).await;
    match employee {
        Ok(employee) => {
            let hashed_password = employee.password.unwrap();
            let password_verified =
                password_utils::verify_hashed_password(password, hashed_password).await;
            if password_verified {
                Ok(true)
            } else {
                Err((StatusCode::UNAUTHORIZED, "Invalid credentials"))
            }
        }
        Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid credentials")),
    }
}

async fn verify_admin_password(
    id: String,
    password: String,
) -> Result<bool, (StatusCode, &'static str)> {
    let admin = get_admin_by_id(id.clone()).await;
    match admin {
        Ok(admin) => {
            if admin.id.is_empty() && admin.password.is_none() {
                Err((StatusCode::UNAUTHORIZED, "Invalid credentials"))
            } else {
                let random_password = admin.password.unwrap();
                if password == random_password {
                    Ok(true)
                } else {
                    Err((StatusCode::UNAUTHORIZED, "Invalid credentials"))
                }
            }
        }
        Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid credentials")),
    }
}

async fn list_employees_renderer(mut context: Context, templates: Arc<Tera>) -> Html<String> {
    let query_options = QueryOptions {
        page: Some(1),
        limit: Some(1000),
    };
    let opts: Option<Query<QueryOptions>> = Some(Query(query_options));

    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let result = list().await;

    match result {
        Ok(employees_map) => {
            let filtered_employees: HashMap<String, Employee> =
                employees_map.into_iter().skip(offset).take(limit).collect();
            let mut vec_employees: Vec<Employee> = filtered_employees.values().cloned().collect();
            vec_employees.sort_by(|x, y: &Employee| x.first_name.cmp(&y.first_name));
            context.insert("employees", &vec_employees);
            Html(templates.render("dashboard.html", &context).unwrap())
        }
        Err(_) => {
            let error_response = EmployeeErrorResponse {
                error: "Error fetching employees".to_string(),
            };
            error!("{error_response:?}");
            context.insert("error_message", &error_response);
            Html(templates.render("errors.html", &context).unwrap())
        }
    }
}

pub async fn edit_employee(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let token_valid = validate_session_token(state, "admin".to_string()).await;
    match token_valid {
        true => {
            let mut context = Context::new();
            context.insert("title", "Edit Employee");

            let result = list().await;
            match result {
                Ok(employee_response) => {
                    let employee = employee_response.get(&id).unwrap();
                    context.insert("employee", &employee);
                    Html(templates.render("edit_form.html", &context).unwrap())
                }
                Err(_) => {
                    let error_response = EmployeeErrorResponse {
                        error: "Employee not found".to_string(),
                    };
                    error!("{error_response:?}");
                    context.insert("error_message", &error_response);
                    Html(templates.render("errors.html", &context).unwrap())
                }
            }
        }
        false => {
            let mut context = Context::new();
            context.insert("title", "Login to Avaya Red Carpet");

            Html(templates.render("login.html", &context).unwrap())
        }
    }
}

pub async fn delete_employee(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let token_valid = validate_session_token(state, "admin".to_string()).await;
    match token_valid {
        true => {
            let mut context = Context::new();
            context.insert("title", "Delete Employee");

            let result = delete(id.clone()).await;
            match result {
                Ok(employees_map) => {
                    let mut vec_employees: Vec<Employee> =
                        employees_map.values().cloned().collect();
                    vec_employees.sort_by(|x, y| x.first_name.cmp(&y.first_name));

                    context.insert("employees", &vec_employees);
                    Html(templates.render("dashboard.html", &context).unwrap())
                }
                Err(_) => {
                    let error_response = EmployeeErrorResponse {
                        error: "Error fetching employees".to_string(),
                    };
                    error!("{error_response:?}");
                    context.insert("error_message", &error_response);
                    Html(templates.render("errors.html", &context).unwrap())
                }
            }
        }
        false => {
            let mut context = Context::new();
            context.insert("title", "Login to Avaya Red Carpet");

            Html(templates.render("login.html", &context).unwrap())
        }
    }
}

pub async fn select_employee(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let token_valid = validate_session_token(state, "admin".to_string()).await;
    match token_valid {
        true => {
            let mut context = Context::new();
            context.insert("title", "Employee");

            let employee_result = get_employee_by_id(id.clone()).await;
            match employee_result {
                Ok(employee) => {
                    context.insert("employee", &employee);
                    Html(templates.render("employee.html", &context).unwrap())
                }
                Err(_) => {
                    let error_response = EmployeeErrorResponse {
                        error: "Employee not found".to_string(),
                    };
                    error!("{error_response:?}");
                    context.insert("error_message", &error_response);
                    Html(templates.render("errors.html", &context).unwrap())
                }
            }
        }
        false => {
            let mut context = Context::new();
            context.insert("title", "Login to Avaya Red Carpet");

            Html(templates.render("login.html", &context).unwrap())
        }
    }
}

pub async fn handle_edit_form_data(
    Extension(templates): Extension<Templates>,
    Form(modified_employee_data): Form<Employee>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Edit Employee");

    let employees_map = update(modified_employee_data.clone()).await;
    match employees_map {
        Ok(employees) => {
            let mut vec_employees: Vec<Employee> = employees.values().cloned().collect();
            vec_employees.sort_by(|x, y| x.first_name.cmp(&y.first_name));

            context.insert("employees", &vec_employees);
            Html(templates.render("dashboard.html", &context).unwrap())
        }
        Err(_) => {
            let error_response = EmployeeErrorResponse {
                error: "Error updating employee".to_string(),
            };
            error!("{error_response:?}");
            context.insert("error_message", &error_response);
            Html(templates.render("errors.html", &context).unwrap())
        }
    }
}

pub async fn handle_save_form_data(
    Extension(templates): Extension<Templates>,
    Form(new_employee_data): Form<Employee>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Personal Data");

    let new_employee = Employee {
        id: Some(Uuid::new_v4().to_string()),
        first_name: new_employee_data.first_name,
        last_name: new_employee_data.last_name,
        personal_email: new_employee_data.personal_email,
        avaya_email: None,
        age: new_employee_data.age,
        diploma: new_employee_data.diploma,
        onboarded: Some(false),
        handle: None,
        password: None,
        secure_password: Some(false),
    };

    let save_result = save(new_employee.clone()).await;

    match save_result {
        Ok(saved) => {
            if saved {
                context.insert("employee", &new_employee);
                Html(templates.render("save_result.html", &context).unwrap())
            } else {
                let warning_response = EmployeeErrorResponse {
                    error: "Personal Data already exists!".to_string(),
                };
                warn!("{warning_response:?}");
                context.insert("error_message", &warning_response);
                Html(templates.render("errors.html", &context).unwrap())
            }
        }
        Err(_) => {
            let warning_response = EmployeeErrorResponse {
                error: "Personal Data already exists!".to_string(),
            };
            warn!("{warning_response:?}");
            context.insert("error_message", &warning_response);
            Html(templates.render("errors.html", &context).unwrap())
        }
    }
}

pub async fn handle_onboard_form_data(
    State(state): State<AppState>,
    Extension(templates): Extension<Templates>,
    Form(onboarding_employee): Form<Employee>,
) -> impl IntoResponse {
    let token_valid = validate_session_token(state, "admin".to_string()).await;
    match token_valid {
        true => {
            let mut context = Context::new();
            context.insert("title", "Onboarding Employee");
            let new_handle = generate_handle(
                onboarding_employee.first_name.clone(),
                onboarding_employee.last_name.clone(),
            )
            .await;

            let employee = Employee {
                id: onboarding_employee.id.clone(),
                first_name: onboarding_employee.first_name.clone(),
                last_name: onboarding_employee.last_name.clone(),
                personal_email: onboarding_employee.personal_email.clone(),
                avaya_email: Some(format!("{}@avaya.com", new_handle)),
                age: onboarding_employee.age,
                diploma: onboarding_employee.diploma.clone(),
                onboarded: Some(true),
                handle: Some(new_handle),
                password: Some(generate_random_password().await),
                secure_password: Some(false),
            };

            let employees_map = update(employee).await;

            match employees_map {
                Ok(employees) => {
                    let mut vec_employees: Vec<Employee> = employees.values().cloned().collect();
                    vec_employees.sort_by(|x, y| x.first_name.cmp(&y.first_name));

                    context.insert("employees", &vec_employees);
                    Html(templates.render("dashboard.html", &context).unwrap())
                }
                Err(_) => {
                    let error_response = EmployeeErrorResponse {
                        error: "Error onboarding employee".to_string(),
                    };
                    error!("{error_response:?}");
                    context.insert("error_message", &error_response);
                    Html(templates.render("errors.html", &context).unwrap())
                }
            }
        }
        false => {
            let mut context = Context::new();
            context.insert("title", "Login to Avaya Red Carpet");

            Html(templates.render("login.html", &context).unwrap())
        }
    }
}

pub async fn secure_password(
    State(state): State<AppState>,
    Extension(templates): Extension<Templates>,
    Form(employee): Form<Employee>,
) -> impl IntoResponse {
    let token_valid = validate_session_token(state, "admin".to_string()).await;
    match token_valid {
        true => {
            let mut context = Context::new();
            context.insert("title", "Securing Password");

            warn!("employee.handle ---> {:?}", employee.handle);
            warn!("employee.password ---> {:?}", employee.password);

            let hashed_password = hash_password(employee.password.clone().unwrap()).await;
            warn!("hashed_password ---> {:?}", hashed_password);
            let modified_employee = Employee {
                id: employee.id,
                first_name: employee.first_name.clone(),
                last_name: employee.last_name.clone(),
                personal_email: employee.personal_email.clone(),
                avaya_email: employee.avaya_email.clone(),
                age: employee.age,
                diploma: employee.diploma.clone(),
                onboarded: employee.onboarded,
                handle: employee.handle.clone(),
                password: Some(hashed_password),
                secure_password: Some(true),
            };

            let employees_map = update(modified_employee.clone()).await;
            match employees_map {
                Ok(employees) => {
                    let mut vec_employees: Vec<Employee> = employees.values().cloned().collect();
                    vec_employees.sort_by(|x, y| x.first_name.cmp(&y.first_name));

                    context.insert("employees", &vec_employees);
                    Html(templates.render("dashboard.html", &context).unwrap())
                }
                Err(_) => {
                    let error_response = EmployeeErrorResponse {
                        error: "Error securing employee".to_string(),
                    };
                    error!("{error_response:?}");
                    context.insert("error_message", &error_response);
                    Html(templates.render("errors.html", &context).unwrap())
                }
            }
        }
        false => {
            let mut context = Context::new();
            context.insert("title", "Login to Avaya Red Carpet");

            Html(templates.render("login.html", &context).unwrap())
        }
    }
}

pub async fn logout_admin(
    State(state): State<AppState>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    warn!("Logging out admin");
    state.sessions.lock().await.remove("admin");
    let mut context = Context::new();
    context.insert("title", "Login to Avaya Red Carpet");
    Html(templates.render("login.html", &context).unwrap())
}

pub async fn login_admin(
    State(state): State<AppState>,
    Extension(templates): Extension<Templates>,
    Form(admin_login_data): Form<Admin>,
) -> impl IntoResponse {
    let mut context = Context::new();
    if state.sessions.lock().await.contains_key("admin") {
        context.insert("title", "Login to Avaya Red Carpet");
        context.insert("error_message", "Already logged in");
        context.insert("already_logged_in", &true);

        Html(
            templates
                .render("already_logged_in.html", &context)
                .unwrap(),
        )
    } else {
        warn!("admin_login_data---> {:?}", admin_login_data);
        let token = generate_session_token(admin_login_data.id.clone()).await;
        warn!("token---> {:?}", token);

        let new_admin = Admin {
            id: admin_login_data.id.clone(),
            password: admin_login_data.password.clone(),
        };

        let admin_result = get_admin_by_id(new_admin.id.clone()).await;

        match admin_result {
            Ok(admin) => {
                if admin.id.is_empty() && admin.password.is_none() {
                    context.insert("title", "Login to Avaya Red Carpet");
                    context.insert("error_message", "Invalid credentials");
                    Html(templates.render("login.html", &context).unwrap())
                } else if admin.password.unwrap() == new_admin.password.unwrap() {
                    // Store the session token in the state
                    state
                        .sessions
                        .lock()
                        .await
                        .insert("admin".to_string(), token.clone());
                    context.insert("title", "Admin Dashboard");
                    list_employees_renderer(context, templates).await
                } else {
                    context.insert("title", "Login");
                    context.insert("error_message", "Invalid credentials");
                    Html(templates.render("login.html", &context).unwrap())
                }
            }
            Err(_) => {
                context.insert("title", "Login");
                context.insert("error_message", "Invalid credentials");
                Html(templates.render("login.html", &context).unwrap())
            }
        }
    }
}

//
// REST /api/v1 related handlers
//

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
    AuthBasic((id, password)): AuthBasic,
    Json(body): Json<EmployeeRequestBody>,
) -> Result<Json<EmployeeResponse>, (StatusCode, Json<EmployeeErrorResponse>)> {
    let verified_result = verify_admin_password(id, password.unwrap()).await;

    match verified_result {
        Ok(verified) => {
            if verified {
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
                    Ok(saved) => {
                        if saved {
                            let json_response = EmployeeResponse {
                                message: "Employee created successfully".to_string(),
                                data: employee,
                            };
                            debug!("{json_response:?}");
                            Ok(Json(json_response))
                        } else {
                            let error_response = EmployeeErrorResponse {
                                error: "Employee already exists or Employee age must be greater than 18 and diploma must not be empty".to_string(),
                            };
                            warn!("{error_response:?}");
                            Err((StatusCode::ALREADY_REPORTED, Json(error_response)))
                        }
                    }
                    Err(error) => {
                        debug!("{error:?}");
                        let error_response = EmployeeErrorResponse {
                            error: error.to_string(),
                        };
                        warn!("{error_response:?}");
                        Err((StatusCode::NOT_MODIFIED, Json(error_response)))
                    }
                }
            } else {
                let error_response = EmployeeErrorResponse {
                    error: "Invalid credentials".to_string(),
                };
                warn!("{error_response:?}");
                Err((StatusCode::UNAUTHORIZED, Json(error_response)))
            }
        }
        Err(error) => {
            error!("{error:?}");
            let error_response = EmployeeErrorResponse {
                error: "Invalid credentials".to_string(),
            };
            Err((StatusCode::UNAUTHORIZED, Json(error_response)))
        }
    }
}

pub async fn employees_list(
    AuthBasic((id, password)): AuthBasic,
    opts: Option<Query<QueryOptions>>,
) -> Result<Json<EmployeeListResponse>, (StatusCode, Json<EmployeeErrorResponse>)> {
    let verified_result = verify_admin_password(id, password.unwrap()).await;

    match verified_result {
        Ok(verified) => {
            if verified {
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

                        let mut vec_employees: Vec<Employee> =
                            filtered_employees.values().cloned().collect();
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
            } else {
                let error_response = EmployeeErrorResponse {
                    error: "Invalid credentials".to_string(),
                };
                Err((StatusCode::UNAUTHORIZED, Json(error_response)))
            }
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                error: "Invalid credentials".to_string(),
            };
            Err((StatusCode::UNAUTHORIZED, Json(error_response)))
        }
    }
}

pub async fn get_employee(
    AuthBasic((id, password)): AuthBasic,
    Path(emp_id): Path<String>,
) -> Result<Json<EmployeeResponse>, (StatusCode, Json<EmployeeErrorResponse>)> {
    let verified_result = verify_admin_password(id, password.unwrap()).await;

    match verified_result {
        Ok(verified) => {
            if verified {
                let employee_result = get_employee_by_id(emp_id.clone()).await;
                match employee_result {
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
                        Err((StatusCode::NOT_FOUND, Json(error_response)))
                    }
                }
            } else {
                let error_response = EmployeeErrorResponse {
                    error: "Invalid credentials".to_string(),
                };
                Err((StatusCode::UNAUTHORIZED, Json(error_response)))
            }
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                error: "Invalid credentials".to_string(),
            };
            Err((StatusCode::UNAUTHORIZED, Json(error_response)))
        }
    }
}

pub async fn employee_by_handle(
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

pub async fn generate_handle_and_password(
    AuthBasic((id, password)): AuthBasic,
    Path(emp_id): Path<String>,
) -> impl IntoResponse {
    let verified_result = verify_admin_password(id, password.unwrap()).await;

    match verified_result {
        Ok(verified) => {
            if verified {
                let employees_list = list().await;

                match employees_list {
                    Ok(employees) => {
                        // list not onboarded employees

                        let filtered_employee = employees.get(&emp_id).unwrap();

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
            } else {
                let error_response = EmployeeErrorResponse {
                    error: "Invalid credentials".to_string(),
                };
                Err((StatusCode::UNAUTHORIZED, Json(error_response)))
            }
        }
        Err(error) => {
            debug!("{error:?}");
            let error_response = EmployeeErrorResponse {
                error: "Invalid credentials".to_string(),
            };
            Err((StatusCode::UNAUTHORIZED, Json(error_response)))
        }
    }
}

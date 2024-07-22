use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::Path,
    http::{self, Response, StatusCode},
    response::{Html, IntoResponse},
    Extension, Form,
};
use axum_auth::AuthBasic;
use log::{debug, error, warn};
use tera::{Context, Tera};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    database::persistence::{delete, get_employee_by_handle, list, save, update},
    models::models::{Employee, EmployeeErrorResponse},
    utils::{state::{LoggedInState, User}, utils::{
        generate_handle, generate_random_password, hash_password, verify_hashed_password,
    }},
};

type Templates = Arc<Tera>;

pub async fn styles() -> impl IntoResponse {
    Response::builder()
        .status(http::StatusCode::OK)
        .header("Content-Type", "text/css")
        .body(include_str!("./public/styles.css").to_owned())
        .unwrap()
}

pub async fn index(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Welcome to Avaya Red Carpet");

    Html(templates.render("index.html", &context).unwrap())
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



async fn sort_by_first_name(
    employees_map: Result<HashMap<String, Employee>, std::io::Error>,
    mut context: Context,
    templates: Arc<Tera>,
) -> Html<String> {
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

            context.insert("error_message", &error_response);
            Html(templates.render("index.html", &context).unwrap())
        }
    }
}

pub async fn list_employees(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "List Employees");
    context.insert("selected_id", "");

    let employees_map = list().await;
    sort_by_first_name(employees_map, context, templates).await
}

pub async fn edit_employee(
    Path(id): Path<String>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Employee");

    let employees_map = list().await;

    filter_by_id(employees_map, id, "edit_form.html", context, templates).await
}

async fn filter_by_id(
    employees_map: Result<HashMap<String, Employee>, std::io::Error>,
    id: String,
    tempate_name: &str,
    mut context: Context,
    templates: Arc<Tera>,
) -> Html<String> {
    match employees_map {
        Ok(employees) => {
            // list not onboarded employees
            let filtered_employees: HashMap<String, Employee> = employees
                .into_iter()
                .filter(|(_id, employee)| employee.id == Some(id.clone()))
                .collect();

            let employees_list: Vec<Employee> = filtered_employees.into_values().collect();

            let employee: &Employee = employees_list.first().unwrap();
            debug!("{employee:?}");

            context.insert("employee", &employee);

            Html(templates.render(tempate_name, &context).unwrap())
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

    let employees_map = delete(id).await;

    sort_by_first_name(employees_map, context, templates).await
}

pub async fn select_employee(
    Path(id): Path<String>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Employee");

    let employees_map = list().await;
    filter_by_id(employees_map, id, "employee.html", context, templates).await
}

pub async fn handle_edit_form_data(
    Extension(templates): Extension<Templates>,
    Form(modified_employee_data): Form<Employee>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Edit Employee");

    let modified_employee = Employee {
        id: modified_employee_data.id,
        first_name: modified_employee_data.first_name.clone(),
        last_name: modified_employee_data.last_name.clone(),
        personal_email: modified_employee_data.personal_email.clone(),
        avaya_email: modified_employee_data.avaya_email.clone(),
        age: modified_employee_data.age,
        diploma: modified_employee_data.diploma.clone(),
        onboarded: modified_employee_data.onboarded,
        handle: modified_employee_data.handle.clone(),
        password: modified_employee_data.password.clone(),
        secure_password: modified_employee_data.secure_password.clone(),
    };

    debug!("modified_employee ---> {modified_employee:?}");
    let employees_map = update(modified_employee).await;

    sort_by_first_name(employees_map, context, templates).await
}

pub async fn handle_save_form_data(
    Extension(templates): Extension<Templates>,
    Form(new_employee_data): Form<Employee>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Personal Data");
    let id = Some(Uuid::new_v4().to_string());

    let new_employee = Employee {
        id,
        first_name: new_employee_data.first_name.clone(),
        last_name: new_employee_data.last_name.clone(),
        personal_email: new_employee_data.personal_email.clone(),
        avaya_email: new_employee_data.avaya_email.clone(),
        age: new_employee_data.age,
        diploma: new_employee_data.diploma.clone(),
        onboarded: Some(false),
        handle: None,
        password: None,
        secure_password: Some(false),
    };

    debug!("new_employee ---> {new_employee:?}");
    let save_result = save(new_employee.clone()).await;

    match save_result {
        Ok(result) => {
            if result {
                context.insert("employee", &new_employee);

                Html(templates.render("save_result.html", &context).unwrap())
            } else {
                let warning_response = EmployeeErrorResponse {
                    status: "Warning".to_string(),
                    description: "Personal Data already exists!".to_string(),
                };
                warn!("{warning_response:?}");
                context.insert("error_message", &warning_response);
                Html(templates.render("errors.html", &context).unwrap())
            }
        }
        Err(_) => {
            let error_response = EmployeeErrorResponse {
                status: "Error".to_string(),
                description: "Employee already exists".to_string(),
            };
            error!("{error_response:?}");
            context.insert("error_message", &error_response);
            Html(templates.render("errors.html", &context).unwrap())
        }
    }
}

pub async fn handle_onboard_form_data(
    Extension(templates): Extension<Templates>,
    Form(onboarding_employee): Form<Employee>,
) -> impl IntoResponse {
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

    debug!("onboarding_employee ---> {:?}", onboarding_employee);
    let employees_map = update(employee.clone()).await;

    sort_by_first_name(employees_map, context, templates).await
}

pub async fn secure_password(
    Extension(templates): Extension<Templates>,
    Form(employee): Form<Employee>,
) -> impl IntoResponse {
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

    let employees_map = update(modified_employee).await;

    sort_by_first_name(employees_map, context, templates).await
}

async fn verify_password(
    handle: String,
    password: String,
) -> LoggedInState {
    let mut context = Context::new();
    context.insert("title", "Edit Employee");

    let employee_by_handle = get_employee_by_handle(handle.clone()).await;

    match employee_by_handle {
        Ok(employee) => {
            let hashed_password = employee.password.unwrap();
            let password_verified = verify_hashed_password(password.clone(), hashed_password).await;
            if password_verified {
                let user = Some(User {
                    handle: handle.clone(),
                    password: password,
                });
                return LoggedInState { user };
            } else {
                return LoggedInState { user: None };
            }
    
        }
        Err(_) => {
            let error_response = EmployeeErrorResponse {
                status: "Error".to_string(),
                description: "Employee already exists".to_string(),
            };
            error!("{error_response:?}");
           
            return LoggedInState { user: None };
        }
    }
}

pub async fn basic_auth_handler(
    AuthBasic((handle, password)): AuthBasic,
    Extension(state): Extension<Arc<Mutex<LoggedInState>>>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let mut logged_in_state = state.lock().await;
    *logged_in_state = verify_password(handle.clone(), password.unwrap()).await;

    // Example validation: Check if the username is "admin" and the password matches the state's secret_key
    if handle == "admin" && password == Some(state.user.as_ref().unwrap().password.clone()) {
        "You are authenticated".into_response()
    } else {
        (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
    }
}
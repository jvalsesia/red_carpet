use std::sync::Arc;

use axum::{
    extract::{Path, Query},
    http::{self, Response},
    response::{Html, IntoResponse},
    Extension, Form, Json,
};
use log::{debug, error, warn};

use tera::{Context, Tera};

use crate::{
    backend::api::{
        create_employee, delete_employee_by_id, employees_list, get_employee, update_employee,
    },
    models::employee_models::{Employee, EmployeeErrorResponse, EmployeeRequestBody, QueryOptions},
    utils::password_utils::{generate_handle, generate_random_password, hash_password},
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

pub async fn list_employees(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "List Employees");
    context.insert("selected_id", "");

    let query_options = QueryOptions {
        page: Some(1),
        limit: Some(1000),
    };
    let opts: Option<Query<QueryOptions>> = Some(Query(query_options));

    let employees_map = employees_list(opts).await;
    match employees_map {
        Ok(result) => {
            let employees = result.0.employees;
            context.insert("employees", &employees);
            Html(templates.render("employees.html", &context).unwrap())
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
    // sort_by_first_name(employees_map, context, templates).await
}

pub async fn edit_employee(
    Path(id): Path<String>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Employee");

    let path: Path<String> = Path(id.clone());
    let result = get_employee(path).await;
    match result {
        Ok(employee_response) => {
            let employee = employee_response.0.data;
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

pub async fn delete_employee(
    Path(id): Path<String>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Edit Employee");

    let path: Path<String> = Path(id.clone());
    let result = delete_employee_by_id(path).await;
    match result {
        Ok(result) => {
            let employees = result.0.employees;
            context.insert("employees", &employees);
            Html(templates.render("employees.html", &context).unwrap())
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

pub async fn select_employee(
    Path(id): Path<String>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Employee");

    let path: Path<String> = Path(id.clone());
    let result = get_employee(path).await;
    match result {
        Ok(employee_response) => {
            let employee = employee_response.0.data;
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

pub async fn handle_edit_form_data(
    Extension(templates): Extension<Templates>,
    Form(modified_employee_data): Form<Employee>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Edit Employee");
    let result = update_employee(Json(modified_employee_data)).await;
    match result {
        Ok(result) => {
            let employees = result.0.employees;
            context.insert("employees", &employees);
            Html(templates.render("employees.html", &context).unwrap())
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

    let new_employee = EmployeeRequestBody {
        first_name: new_employee_data.first_name.clone(),
        last_name: new_employee_data.last_name.clone(),
        personal_email: new_employee_data.personal_email.clone(),
        age: new_employee_data.age,
        diploma: new_employee_data.diploma.clone(),
    };

    let create_result = create_employee(Json(new_employee)).await;
    match create_result {
        Ok(result) => {
            let employee_response = result.0;
            let new_employee = employee_response.data;
            context.insert("employee", &new_employee);
            debug!("new_employee ---> {new_employee:?}");
            Html(templates.render("save_result.html", &context).unwrap())
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

    let result = update_employee(Json(employee)).await;
    match result {
        Ok(result) => {
            let employees = result.0.employees;
            context.insert("employees", &employees);
            Html(templates.render("employees.html", &context).unwrap())
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

    let result = update_employee(Json(modified_employee)).await;
    match result {
        Ok(result) => {
            let employees = result.0.employees;
            context.insert("employees", &employees);
            Html(templates.render("employees.html", &context).unwrap())
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

// async fn verify_password(handle: String, password: String) -> LoggedInState {
//     let mut context = Context::new();
//     context.insert("title", "Edit Employee");

//     let path: Path<String> = Path(handle.clone());
//     let employee_by_handle = get_employee_by_handle_2(path).await;

//     match employee_by_handle {
//         Ok(employee_response) => {
//             let employee = employee_response.0.data;
//             let hashed_password = employee.password.unwrap();
//             let is_valid = verify_hashed_password(password, hashed_password.clone()).await;
//             if is_valid {
//                 LoggedInState {
//                     user: Some(User {
//                         handle: employee.handle.unwrap(),
//                         password: hashed_password,
//                     }),
//                 }
//             } else {
//                 LoggedInState { user: None }
//             }
//         }
//         Err(_) => LoggedInState { user: None },
//     }
// }

// pub async fn basic_auth_handler(
//     AuthBasic((handle, password)): AuthBasic,
//     Extension(state): Extension<Arc<Mutex<LoggedInState>>>,
//     Extension(templates): Extension<Templates>,
// ) -> impl IntoResponse {
//     let mut logged_in_state = state.lock().await;
//     *logged_in_state = verify_password(handle.clone(), password.unwrap()).await;

//     // Example validation: Check if the username is "admin" and the password matches the state's secret_key
//     if handle == "admin" && password == Some(state.lock().await.user.as_ref().unwrap().password.clone()) {
//         "You are authenticated".into_response()
//     } else {
//         (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
//     }
// }

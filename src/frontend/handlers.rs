use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::Path,
    http::{self, Response},
    response::{Html, IntoResponse},
    Extension, Form,
};
use log::{debug, error, warn};
use tera::{Context, Tera};
use uuid::Uuid;

use crate::{
    database::persistence::{delete, list, save, update},
    models::models::{Employee, EmployeeErrorResponse},
    utils::utils::{
        generate_handle, generate_random_password, hash_password, verify_hashed_password,
    },
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

pub async fn home(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Welcome to Avaya Red Carpet");

    Html(templates.render("home.html", &context).unwrap())
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

            context.insert("error", &error_response);
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

            let employee = employees_list.first().unwrap();
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
    debug!(
        "modified_employee_data.id ---> {:?}",
        modified_employee_data.id
    );
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
    context.insert("title", "Edit Employee");
    let id = Some(Uuid::new_v4().to_string());
    debug!("new_employee_data.id ---> {:?}", id);
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
                    status: "warning".to_string(),
                    description: "Employee already exists".to_string(),
                };
                warn!("{warning_response:?}");
                context.insert("error", &warning_response);
                Html(templates.render("index.html", &context).unwrap())
            }
        }
        Err(_) => {
            let error_response = EmployeeErrorResponse {
                status: "error".to_string(),
                description: "Employee already exists".to_string(),
            };
            error!("{error_response:?}");
            context.insert("error", &error_response);
            Html(templates.render("index.html", &context).unwrap())
        }
    }
}

pub async fn handle_onboard_form_data(
    Extension(templates): Extension<Templates>,
    Form(onboarding_employee): Form<Employee>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Edit Employee");
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
    context.insert("title", "Edit Employee");
    debug!("modified_employee_data.id ---> {:?}", employee.id);

    let hashed_password = hash_password(employee.password.clone().unwrap()).await;

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
    };

    debug!("modified_employee ---> {modified_employee:?}");
    let employees_map = update(modified_employee).await;

    sort_by_first_name(employees_map, context, templates).await
}

pub async fn verify_password(
    Extension(templates): Extension<Templates>,
    password: String,
    hashed_password: String,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Edit Employee");

    let password_verified = verify_hashed_password(password.clone(), hashed_password.clone()).await;

    if password == hashed_password {
        Html(templates.render("onboard.html", &context).unwrap())
    } else {
        let error_response = EmployeeErrorResponse {
            status: "error".to_string(),
            description: "Invalid password".to_string(),
        };
        error!("{error_response:?}");
        context.insert("error", &error_response);
        Html(templates.render("index.html", &context).unwrap())
    }
}

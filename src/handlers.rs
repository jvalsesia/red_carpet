use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::Path,
    http::{self, Response},
    response::{Html, IntoResponse},
    Extension, Form,
};
use log::debug;
use tera::{Context, Tera};
use uuid::Uuid;

use crate::{
    models::{Employee, EmployeeErrorResponse},
    persistence::{delete, list, save, update},
};

type Templates = Arc<Tera>;

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

pub async fn save_success_page(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "Personal Details");

    Html(templates.render("save_success.html", &context).unwrap())
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
    context.insert("title", "Employee");

    let employees_map = list().await;

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

    let employees_map = delete(id).await;

    match employees_map {
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
    context.insert("title", "Employee");

    let employees_map = list().await;
    match employees_map {
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
        email: new_employee_data.email.clone(),
        age: new_employee_data.age,
        diploma: new_employee_data.diploma.clone(),
        onboarded: Some(false),
        handle: None,
        password: None,
    };

    debug!("new_employee ---> {new_employee:?}");
    let save_result = save(new_employee.clone()).await;

    match save_result {
        Ok(employee) => {
            context.insert("employee", &new_employee);

            Html(templates.render("save_success.html", &context).unwrap())
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

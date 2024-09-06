use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Employee {
    pub id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub personal_email: Option<String>,
    pub avaya_email: Option<String>,
    pub age: u32,
    pub diploma: String,
    pub onboarded: Option<bool>,
    pub handle: Option<String>,
    pub password: Option<String>,
    pub secure_password: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EmployeeForm {
    pub handle: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmployeeRequestBody {
    pub first_name: String,
    pub last_name: String,
    pub personal_email: Option<String>,
    pub age: u32,
    pub diploma: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmployeeData {
    pub employee: Employee,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmployeeResponse {
    pub message: String,
    pub data: Employee,
}

#[derive(Serialize, Debug)]
pub struct EmployeeListResponse {
    pub message: String,
    pub results: usize,
    pub employees: Vec<Employee>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateEmployeeSchema {
    pub title: Option<String>,
    pub content: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct QueryOptions {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmployeeErrorResponse {
    pub error: String,
}

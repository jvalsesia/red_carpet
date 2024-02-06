use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Employee {
    #[serde(skip_deserializing)]
    pub id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_deserializing)]
    pub email: String,
    pub age: u32,
    pub diploma: String,
    #[serde(skip_deserializing)]
    pub onboarded: Option<bool>,
}

#[derive(Serialize, Debug)]
pub struct EmployeeData {
    pub employee: Employee,
}

#[derive(Serialize, Debug)]
pub struct SimpleEmployeeResponse {
    pub status: String,
    pub data: EmployeeData,
}

#[derive(Serialize, Debug)]
pub struct EmployeeListResponse {
    pub status: String,
    pub results: usize,
    pub todos: Vec<Employee>,
}

pub type DB = Arc<Mutex<Vec<Employee>>>;

pub fn employee_db() -> DB {
    Arc::new(Mutex::new(Vec::new()))
}

#[derive(Debug, Deserialize, Default)]
pub struct QueryOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

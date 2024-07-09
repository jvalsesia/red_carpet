use std::sync::Arc;

use tokio::sync::Mutex;

use crate::models::models::Employee;

pub type DB = Arc<Mutex<Vec<Employee>>>;

pub fn employee_db() -> DB {
    Arc::new(Mutex::new(Vec::new()))
}

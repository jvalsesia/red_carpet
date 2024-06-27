use log::{debug, info};

use crate::models::Employee;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Result,
    path::Path,
};

const DATA_DIR: &str = "data";
const DATA_FILE: &str = "data/employees.json";

pub fn create_persistence_store() -> Result<()> {
    if Path::new(DATA_DIR).exists() {
        info!("Persistence directory already exists: {DATA_DIR:?}");
    } else {
        info!("Creating Persistence directory : {DATA_DIR:?}");
        let _d = fs::create_dir_all(DATA_DIR)?;
    }

    if Path::new(DATA_FILE).exists() {
        info!("Persistence file already exists: {DATA_FILE:?}");
    } else {
        info!("Creating Persistence file : {DATA_FILE:?}");
        let _f = File::create(DATA_FILE)?;
    }

    Ok(())
}

pub async fn save(employee: Employee) -> Result<String> {
    let employee_file_path = Path::new(DATA_FILE);
    let data = fs::read_to_string(employee_file_path).expect("Unable to read file");
    let mut map_employees: HashMap<String, Employee> = HashMap::new();

    //let mut employees: Vec<Employee> = Vec::new();
    if fs::metadata(employee_file_path).unwrap().len() != 0 {
        //  employees = serde_json::from_str(&data)?;
        map_employees = serde_json::from_str(&data)?;
    }
    map_employees.insert(employee.id.clone().unwrap(), employee.clone());
    // employees.push(employee.clone());

    let json: String = serde_json::to_string_pretty(&map_employees)?;
    fs::write(employee_file_path, &json).expect("Unable to write file");
    debug!("saving employee: {employee:?}");

    Ok(json)
}

pub async fn update(modified_employee: Employee) -> Result<HashMap<String, Employee>> {
    let employee_file_path = Path::new(DATA_FILE);
    let data = fs::read_to_string(employee_file_path).expect("Unable to read file");
    let mut map_employees: HashMap<String, Employee> = HashMap::new();
    debug!("modified_employee: {modified_employee:?}");
    //let mut employees: Vec<Employee> = Vec::new();
    if fs::metadata(employee_file_path).unwrap().len() != 0 {
        //  employees = serde_json::from_str(&data)?;
        map_employees = serde_json::from_str(&data)?;
    }

    map_employees
        .entry(modified_employee.id.clone().unwrap())
        .and_modify(|employee| *employee = modified_employee.clone())
        .or_insert(modified_employee.clone());

    let json: String = serde_json::to_string_pretty(&map_employees)?;
    fs::write(employee_file_path, &json).expect("Unable to write file");
    debug!("updated employee: {modified_employee:?}");

    Ok(map_employees)
}

pub async fn delete(id: String) -> Result<HashMap<String, Employee>> {
    let employee_file_path = Path::new(DATA_FILE);
    let data = fs::read_to_string(employee_file_path).expect("Unable to read file");
    let mut map_employees: HashMap<String, Employee> = HashMap::new();

    //let mut employees: Vec<Employee> = Vec::new();
    if fs::metadata(employee_file_path).unwrap().len() != 0 {
        //  employees = serde_json::from_str(&data)?;
        map_employees = serde_json::from_str(&data)?;
    }

    let removed_employee = map_employees.get(&id);
    debug!("removing employee: {removed_employee:?}");
    map_employees.remove(&id);

    // employees.push(employee.clone());

    let json: String = serde_json::to_string_pretty(&map_employees)?;
    fs::write(employee_file_path, &json).expect("Unable to write file");

    Ok(map_employees)
}

pub async fn list() -> Result<HashMap<String, Employee>> {
    let employee_file_path = Path::new(DATA_FILE);

    let data = fs::read_to_string(employee_file_path).expect("Unable to read file");

    let mut map_employees: HashMap<String, Employee> = HashMap::new();

    //let mut employees: Vec<Employee> = Vec::new();
    if fs::metadata(employee_file_path).unwrap().len() != 0 {
        //employees = serde_json::from_str(&data)?;
        map_employees = serde_json::from_str(&data)?;
    }

    let json: String = serde_json::to_string_pretty(&map_employees)?;
    fs::write(employee_file_path, &json).expect("Unable to write file");

    Ok(map_employees)
}

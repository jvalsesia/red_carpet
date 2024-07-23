use log::{debug, info};

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Result,
    path::Path,
};

use crate::{
    models::{admin_models::Admin, employee_models::Employee},
    utils::password_utils::generate_random_password,
};

const DATA_DIR: &str = "data";
const ADMIN_DATA_FILE: &str = "data/admin.json";
const EMPLOYEE_DATA_FILE: &str = "data/employees.json";

pub fn create_persistence_store() -> Result<()> {
    if Path::new(DATA_DIR).exists() {
        info!("Persistence directory already exists: {DATA_DIR:?}");
    } else {
        info!("Creating Persistence directory : {DATA_DIR:?}");
        fs::create_dir_all(DATA_DIR)?
    }

    if Path::new(ADMIN_DATA_FILE).exists() {
        info!("Admin Persistence file already exists: {ADMIN_DATA_FILE:?}");
    } else {
        info!("Creating Admin Persistence file : {ADMIN_DATA_FILE:?}");
        File::create(ADMIN_DATA_FILE)?;
    }

    if Path::new(EMPLOYEE_DATA_FILE).exists() {
        info!("Employee Persistence file already exists: {EMPLOYEE_DATA_FILE:?}");
    } else {
        info!("Creating Employee Persistence file : {EMPLOYEE_DATA_FILE:?}");
        File::create(EMPLOYEE_DATA_FILE)?;
    }

    Ok(())
}

pub async fn create_admin(admin: Admin) -> Result<bool> {
    let admin_exists = check_admin_exists(admin.id.clone()).await;

    match admin_exists {
        Ok(result) => {
            if result {
                debug!("Admin already exists");
                Ok(true)
            } else {
                let admin_file_path = Path::new(ADMIN_DATA_FILE);
                let data = fs::read_to_string(admin_file_path).expect("Unable to read file");
                let mut map_admins: HashMap<String, Admin> = HashMap::new();

                //let mut admins: Vec<Admin> = Vec::new();
                if fs::metadata(admin_file_path).unwrap().len() != 0 {
                    //  admins = serde_json::from_str(&data)?;
                    map_admins = serde_json::from_str(&data)?;
                }
                let password = generate_random_password().await;

                let new_admin = Admin {
                    id: admin.id.clone(),
                    password: Some(password.clone()),
                };

                map_admins.insert(admin.id.clone(), new_admin.clone());
                // admins.push(admin.clone());

                let json: String = serde_json::to_string_pretty(&map_admins)?;
                fs::write(admin_file_path, json).expect("Unable to write file");
                debug!("saving admin: {new_admin:?}");

                Ok(false)
            }
        }
        Err(_) => Ok(false),
    }
}

pub async fn save(employee: Employee) -> Result<bool> {
    let employee_exists =
        check_employee_exists(employee.first_name.clone(), employee.last_name.clone()).await;

    match employee_exists {
        Ok(result) => {
            if result {
                debug!("Employee already exists");
                Ok(false)
            } else {
                let employee_file_path = Path::new(EMPLOYEE_DATA_FILE);
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
                fs::write(employee_file_path, json).expect("Unable to write file");
                debug!("saving employee: {employee:?}");

                Ok(true)
            }
        }
        Err(_) => Ok(false),
    }
}

pub async fn update(modified_employee: Employee) -> Result<HashMap<String, Employee>> {
    let employee_file_path = Path::new(EMPLOYEE_DATA_FILE);
    let data = fs::read_to_string(employee_file_path).expect("Unable to read file");
    let mut map_employees: HashMap<String, Employee> = HashMap::new();
    debug!("modified_employee: {modified_employee:?}");
    //let mut employees: Vec<Employee> = Vec::new();
    if fs::metadata(employee_file_path).unwrap().len() != 0 {
        map_employees = serde_json::from_str(&data)?;
    }

    map_employees
        .entry(modified_employee.id.clone().unwrap())
        .and_modify(|employee| *employee = modified_employee.clone())
        .or_insert(modified_employee.clone());

    let json: String = serde_json::to_string_pretty(&map_employees)?;
    fs::write(employee_file_path, json).expect("Unable to write file");
    debug!("updated employee: {modified_employee:?}");

    Ok(map_employees)
}

pub async fn delete(id: String) -> Result<HashMap<String, Employee>> {
    let employee_file_path = Path::new(EMPLOYEE_DATA_FILE);
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
    fs::write(employee_file_path, json).expect("Unable to write file");

    Ok(map_employees)
}

pub async fn list() -> Result<HashMap<String, Employee>> {
    let employee_file_path = Path::new(EMPLOYEE_DATA_FILE);

    let data = fs::read_to_string(employee_file_path).expect("Unable to read file");

    let mut map_employees: HashMap<String, Employee> = HashMap::new();

    //let mut employees: Vec<Employee> = Vec::new();
    if fs::metadata(employee_file_path).unwrap().len() != 0 {
        //employees = serde_json::from_str(&data)?;
        map_employees = serde_json::from_str(&data)?;
    }

    let json: String = serde_json::to_string_pretty(&map_employees)?;
    fs::write(employee_file_path, json).expect("Unable to write file");

    Ok(map_employees)
}

pub async fn check_employee_exists(first_name: String, last_name: String) -> Result<bool> {
    let employee_file_path = Path::new(EMPLOYEE_DATA_FILE);
    let data = fs::read_to_string(employee_file_path).expect("Unable to read file");
    let mut map_employees: HashMap<String, Employee> = HashMap::new();

    //let mut employees: Vec<Employee> = Vec::new();
    if fs::metadata(employee_file_path).unwrap().len() != 0 {
        //  employees = serde_json::from_str(&data)?;
        map_employees = serde_json::from_str(&data)?;
    }

    let employee_exists = map_employees
        .values()
        .any(|employee| employee.first_name == first_name && employee.last_name == last_name);

    Ok(employee_exists)
}

pub async fn check_admin_exists(id: String) -> Result<bool> {
    let admin_file_path = Path::new(ADMIN_DATA_FILE);
    let data = fs::read_to_string(admin_file_path).expect("Unable to read file");
    let mut map_admins: HashMap<String, Admin> = HashMap::new();

    //let mut admins: Vec<Admin> = Vec::new();
    if fs::metadata(admin_file_path).unwrap().len() != 0 {
        //  admins = serde_json::from_str(&data)?;
        map_admins = serde_json::from_str(&data)?;
    }

    let admin_exists = map_admins.contains_key(&id);

    Ok(admin_exists)
}

pub async fn get_employee_by_handle(handle: String) -> Result<Employee> {
    let employee_file_path = Path::new(EMPLOYEE_DATA_FILE);
    let data = fs::read_to_string(employee_file_path).expect("Unable to read file");
    let map_employees: HashMap<String, Employee> = serde_json::from_str(&data)?;

    let employee = map_employees
        .values()
        .find(|employee| employee.handle == Some(handle.clone()))
        .unwrap()
        .clone();

    Ok(employee)
}

pub async fn get_employee_by_id(id: String) -> Result<Employee> {
    let employee_file_path = Path::new(EMPLOYEE_DATA_FILE);
    let data = fs::read_to_string(employee_file_path).expect("Unable to read file");
    let map_employees: HashMap<String, Employee> = serde_json::from_str(&data)?;

    let employee = map_employees.get(&id).unwrap().clone();

    Ok(employee)
}

pub async fn get_admin_by_id(id: String) -> Result<Admin> {
    let admin_file_path = Path::new(ADMIN_DATA_FILE);
    let data = fs::read_to_string(admin_file_path).expect("Unable to read file");
    let map_admins: HashMap<String, Admin> = serde_json::from_str(&data)?;

    let admin = map_admins.get(&id).unwrap().clone();

    Ok(admin)
}

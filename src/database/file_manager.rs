use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::sync::Mutex;

use log::info;

use crate::models::admin_models::Admin;
use crate::models::employee_models::Employee;

#[derive(Debug)]
pub struct FileManager {
    employees: Mutex<HashMap<String, Employee>>,
    admins: Mutex<HashMap<String, Admin>>,
    employee_file_path: String,
    admin_file_path: String,
}

impl FileManager {
    pub fn new(employee_file_path: &str, admin_file_path: &str) -> io::Result<Self> {
        let employees = Self::load_employees_from_file(employee_file_path)?;
        let admins = Self::load_admins_from_file(admin_file_path)?;
        info!("Loaded {} employees", employees.len());
        info!("Loaded {} admins", admins.len());
        Ok(FileManager {
            employees: Mutex::new(employees),
            admins: Mutex::new(admins),
            employee_file_path: employee_file_path.to_string(),
            admin_file_path: admin_file_path.to_string(),
        })
    }

    fn load_employees_from_file(file_path: &str) -> io::Result<HashMap<String, Employee>> {
        let mut file = OpenOptions::new().read(true).open(file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let employees: HashMap<String, Employee> = serde_json::from_str(&content)?;
        Ok(employees)
    }

    fn load_admins_from_file(file_path: &str) -> io::Result<HashMap<String, Admin>> {
        let mut file = OpenOptions::new().read(true).open(file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let admins: HashMap<String, Admin> = serde_json::from_str(&content)?;
        Ok(admins)
    }

    // get employee by id
    fn get_employee_by_id(&self, id: &str) -> Option<Employee> {
        let employees = self.employees.lock().unwrap();
        employees.get(id).cloned()
    }

    // save employee content to file
    fn save_employee_content_to_file(&self, content: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.employee_file_path)?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        Ok(())
    }

    // save admin content to file
    fn save_admins_content_to_file(&self, content: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.admin_file_path)?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        Ok(())
    }

    // check employee exists by first name and last name
    pub fn check_employee_exists(&self, first_name: &str, last_name: &str) -> bool {
        info!("Checking if employee exists: {} {}", first_name, last_name);
        let employees = self.employees.lock().unwrap();
        employees
            .values()
            .any(|employee| employee.first_name == first_name && employee.last_name == last_name)
    }

    // add employee
    pub fn add_employee(&self, employee: Employee) -> io::Result<()> {
        info!("Adding employee: {:?}", employee);
        let mut employees = self.employees.lock().unwrap();
        employees.insert(employee.id.clone().unwrap(), employee);
        let content = serde_json::to_string_pretty(&*employees)?;
        self.save_employee_content_to_file(&content)
    }

    // add admin
    pub fn add_admin(&self, admin: Admin) -> io::Result<()> {
        info!("Adding admin: {:?}", admin);
        let mut admins = self.admins.lock().unwrap();
        admins.insert(admin.id.clone(), admin);
        let content = serde_json::to_string_pretty(&*admins)?;
        self.save_admins_content_to_file(&content)
    }

    // list employees sorted by first name
    pub fn list_employees(&self) -> Vec<Employee> {
        info!("Listing employees");
        let employees = self.employees.lock().unwrap();
        let mut vec_employees: Vec<Employee> = employees.values().cloned().collect();
        vec_employees.sort_by(|a, b| a.first_name.cmp(&b.first_name));
        vec_employees
    }

    // paginate employees
    pub fn paginate_employees(&self, page: usize, per_page: usize) -> Vec<Employee> {
        info!("Paginating employees");
        let employees = self.employees.lock().unwrap();
        let employees: Vec<Employee> = employees.values().cloned().collect();

        // check if page is out of bounds
        if page < 1 {
            return vec![];
        }

        let start = (page - 1) * per_page;
        let end = start + per_page;

        if start >= employees.len() {
            return vec![];
        }
        if end >= employees.len() {
            return employees[start..].to_vec();
        }
        employees[start..end].to_vec()
    }

    // update employee
    pub fn update_employee(&self, id: &str, to_be_update_employee: Employee) -> io::Result<()> {
        info!("Updating employee: {:?}", to_be_update_employee);
        let mut employees = self.employees.lock().unwrap();

        // insert the updated employee
        let insert_result = employees.insert(id.to_string(), to_be_update_employee);

        // if the employee is not found, return an error
        if insert_result.is_none() {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Employee not found",
            ))
        } else {
            let content = serde_json::to_string_pretty(&*employees)?;
            self.save_employee_content_to_file(&content)
        }
    }

    // get employee by handle
    pub fn get_employee_by_handle(&self, handle: &str) -> Option<Employee> {
        info!("Getting employee by handle: {}", handle);
        let employees = self.employees.lock().unwrap();
        employees
            .values()
            .find(|employee| employee.handle == Some(handle.to_string()))
            .cloned()
    }

    pub fn get_employee(&self, id: &str) -> Option<Employee> {
        info!("Getting employee by id: {}", id);
        self.get_employee_by_id(id)
    }

    pub fn get_admin_by_id(&self, id: &str) -> Option<Admin> {
        info!("Getting admin by id: {}", id);
        let admins = self.admins.lock().unwrap();
        admins.get(id).cloned()
    }
    // delete employee
    pub fn delete_employee(&self, id: &str) -> io::Result<()> {
        info!("Deleting employee by id: {}", id);
        let mut employees = self.employees.lock().unwrap();
        employees.remove(id);
        let content = serde_json::to_string_pretty(&*employees)?;
        self.save_employee_content_to_file(&content)
    }
}

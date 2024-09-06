use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::sync::Mutex;

use log::{debug, info};

use crate::models::admin_models::Admin;
use crate::models::employee_models::{Employee, EmployeeRequestBody};

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

    fn save_admins_to_file(&self) -> io::Result<()> {
        let admins = self.admins.lock().unwrap();
        let content = serde_json::to_string(&*admins)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.admin_file_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    // check employee exists by first name and last name
    pub fn check_employee_exists(&self, first_name: &str, last_name: &str) -> bool {
        let employees = self.employees.lock().unwrap();
        employees
            .values()
            .any(|employee| employee.first_name == first_name && employee.last_name == last_name)
    }

    // add employee
    pub fn add_employee(&self, employee: Employee) -> io::Result<()> {
        debug!("Adding employee: {:?}", employee);
        let mut employees = self.employees.lock().unwrap();
        debug!("Employees: {:?}", employees);
        employees.insert(employee.id.clone().unwrap(), employee);
        debug!("Employees after insert: {:?}", employees);
        let content = serde_json::to_string_pretty(&*employees)?;

        self.save_employee_content_to_file(&content)
    }

    // list employees sorted by first name
    pub fn list_employees(&self) -> Vec<Employee> {
        let employees = self.employees.lock().unwrap();
        let mut vec_employees: Vec<Employee> = employees.values().cloned().collect();
        vec_employees.sort_by(|a, b| a.first_name.cmp(&b.first_name));
        vec_employees
    }

    // paginate employees
    pub fn paginate_employees(&self, page: usize, per_page: usize) -> Vec<Employee> {
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
    pub fn update_employee(
        &self,
        id: &str,
        to_be_update_employee: EmployeeRequestBody,
    ) -> io::Result<()> {
        let mut employees = self.employees.lock().unwrap();
        let employee = employees.get_mut(id);

        // if employee is found, update the fields
        if let Some(employee) = employee {
            employee.id = Some(id.to_string());
            employee.first_name = to_be_update_employee.first_name;
            employee.last_name = to_be_update_employee.last_name;
            employee.personal_email = to_be_update_employee.personal_email;
            employee.age = to_be_update_employee.age;
            employee.diploma = to_be_update_employee.diploma;
            let content = serde_json::to_string_pretty(&*employees)?;
            self.save_employee_content_to_file(&content)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Employee not found",
            ))
        }
    }

    pub fn get_employee(&self, id: &str) -> Option<Employee> {
        self.get_employee_by_id(id)
    }

    pub fn get_admin_by_id(&self, id: &str) -> Option<Admin> {
        let admins = self.admins.lock().unwrap();
        admins.get(id).cloned()
    }
}

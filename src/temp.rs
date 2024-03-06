use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufWriter, Seek, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};
use std::io::Result;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Employee {
    pub uuid: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub age: u32,
}

fn main() {
    println!("Hello, world!");

    //let content = load_employee_json();
    // println content key 0100000000010000 name

    // let uuid = Uuid::parse_str("8586fca2-2db9-4236-9899-633f7fe9aa97").unwrap();

    //println!("{:?}", content.unwrap().get(&Some(uuid)).unwrap().email);

    let employee = Employee {
        uuid: Some(Uuid::new_v4()),
        first_name: "Rob".to_string(),
        last_name: "Halford".to_string(),
        email: "halford@judaspriest.com".to_string(),
        age: 72,
    };

    let content = update_employee_json(employee);
    print!("{content:?}");
}

fn load_employee_json() -> Result<HashMap<Option<Uuid>, Employee>> {
    let employee_file_path = Path::new("data/employee.json");
    let file = File::open(employee_file_path)?;
    // get file metadata
    let metadata = file.metadata()?;
    println!("The File Size is: {:?} Bytes", metadata.len());

    let content: HashMap<Option<Uuid>, Employee> = match serde_json::from_reader(&file) {
        Ok(content) => {
            println!("file loaed success");
            content
        }
        Err(e) => {
            println!("file loaed error: {}", e);
            Err(e)?
        }
    };

    println!("{:?}", content);
    println!("----------- end load -----------");

    Ok(content)
}

fn load_employees_json() -> Result<Vec<Employee>> {
    let employee_file_path = Path::new("data/employee.json");

    let data = fs::read_to_string(employee_file_path).expect("Unable to read file");

    let mut employees: Vec<Employee> = Vec::new();
    if fs::metadata(employee_file_path).unwrap().len() != 0 {
        employees = serde_json::from_str(&data)?;
    }

    let json: String = serde_json::to_string_pretty(&employees)?;
    fs::write(employee_file_path, &json).expect("Unable to write file");
    println!("----------- end load -----------");

    Ok(employees)
}
fn update_employee_json(new_employee: Employee) -> Result<Vec<Employee>> {
    let employee_file_path = Path::new("data/employee.json");

    let data = fs::read_to_string(employee_file_path).expect("Unable to read file");

    let mut employees: Vec<Employee> = Vec::new();
    if fs::metadata(employee_file_path).unwrap().len() != 0 {
        employees = serde_json::from_str(&data)?;
    }
    employees.push(new_employee);

    // // get file metadata
    // let metadata = file.metadata()?;
    // println!("The File Size is: {:?} Bytes", metadata.len());

    // let mut content: HashMap<Option<Uuid>, Employee> = match serde_json::from_reader(&file) {
    //     Ok(content) => {
    //         println!("file loaed success");
    //         content
    //     }
    //     Err(e) => {
    //         println!("file loaed error: {}", e);
    //         Err(e)?
    //     }
    // };

    //let uuid_id = Uuid::new_v4();

    // content.insert(Some(uuid_id), new_employee);

    // println!("{:?}", content);

    //serde_json::to_writer_pretty(&mut file, &blah)?;

    let json: String = serde_json::to_string_pretty(&employees)?;
    fs::write(employee_file_path, &json).expect("Unable to write file");
    println!("----------- end load -----------");

    Ok(employees)
}

use log::{debug, info};

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Result,
    path::Path,
};

use crate::{models::admin_models::Admin, utils::password_utils::generate_random_password};

const DATA_DIR: &str = "data";
pub const ADMIN_DATA_FILE: &str = "data/admin.json";
pub const EMPLOYEE_DATA_FILE: &str = "data/employees.json";

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
                Ok(false)
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

                Ok(true)
            }
        }
        Err(_) => Ok(false),
    }
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

    let admin = map_admins.get(&id);
    info!("admin: {admin:?}");
    let admin_exists = map_admins.contains_key(&id);

    Ok(admin_exists)
}

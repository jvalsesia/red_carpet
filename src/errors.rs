use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum DataStoreError {
    #[error("unknown data store error")]
    Unknown,
    #[error("Employee: '{first_name:?} {last_name:?}' already exists!")]
    EmployeeAlreadyExists {
        first_name: String,
        last_name: String,
    },

    #[error("Employee: '{first_name:?} {last_name:?}' is not old enough, no 18 years old yet!")]
    NoOldEnough {
        first_name: String,
        last_name: String,
    },

    #[error("Employee: '{first_name:?} {last_name:?}' does not have diploma!")]
    NoDiploma {
        first_name: String,
        last_name: String,
    },
}

pub fn employee_already_exists_error(
    first_name: String,
    last_name: String,
) -> Result<(), DataStoreError> {
    Err(DataStoreError::EmployeeAlreadyExists {
        first_name,
        last_name,
    })
}

pub fn employee_not_old_enough_error(
    first_name: String,
    last_name: String,
) -> Result<(), DataStoreError> {
    Err(DataStoreError::NoOldEnough {
        first_name,
        last_name,
    })
}

pub fn employee_no_diploma_error(
    first_name: String,
    last_name: String,
) -> Result<(), DataStoreError> {
    Err(DataStoreError::NoDiploma {
        first_name,
        last_name,
    })
}

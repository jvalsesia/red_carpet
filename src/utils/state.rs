
// Define a struct to represent the logged-in user
#[derive(Debug)]
pub struct User {
    pub handle: String,
    pub password: String,
}

// Define a custom State struct to store the logged-in user information
#[derive(Debug)]
pub struct LoggedInState {
    pub user: Option<User>,
}

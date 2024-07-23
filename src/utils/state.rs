use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct AppState {
    pub user_sessions: Arc<Mutex<HashMap<String, String>>>, // Example: Maps usernames to session tokens
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserData {
    pub username: String,
    // Add more fields as needed
}

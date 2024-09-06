use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::database::file_manager::FileManager;

#[derive(Clone, Debug)]
pub struct AppState {
    pub sessions: Arc<Mutex<HashMap<String, String>>>, // Example: Maps usernames to session tokens
    // add employee manager here
    pub file_manager: Arc<FileManager>,
}

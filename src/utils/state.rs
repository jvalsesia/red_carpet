use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct AppState {
    pub sessions: Arc<Mutex<HashMap<String, String>>>, // Example: Maps usernames to session tokens
}

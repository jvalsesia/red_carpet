use std::time::{SystemTime, UNIX_EPOCH};

use log::warn;
use pbkdf2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use rand::{distributions::Alphanumeric, seq::SliceRandom, Rng};

pub const LOWER_CASE: &str = "abcdefghijklmnopqrstuvxyz";
pub const UPPER_CASE: &str = "ABCDEFGHIJKLMNOPQRSTUVXYZ";
pub const SPECIAL_CHARACTER: &str = "!@#$%&*()_-+=,.:;?/|";
pub const NUMBERS: &str = "1234567890";

async fn password(library: String, size: u32) -> String {
    let l: Vec<char> = library.chars().collect();
    (0..size)
        .map(|_| *l.choose(&mut rand::thread_rng()).unwrap())
        .collect()
}

pub async fn generate_handle(first_name: String, last_name: String) -> String {
    format!(
        "{}{}",
        first_name.to_lowercase().chars().next().unwrap(),
        last_name.to_lowercase()
    )
}

pub async fn generate_random_password() -> String {
    let begin = [LOWER_CASE, SPECIAL_CHARACTER, NUMBERS].concat();
    let password_begin = password(begin, 1).await;

    let middle = [LOWER_CASE, UPPER_CASE, NUMBERS, SPECIAL_CHARACTER].concat();
    let password_middle = password(middle, 7).await;

    let end = [LOWER_CASE, UPPER_CASE, NUMBERS].concat();
    let password_end = password(end, 1).await;

    [password_begin, password_middle, password_end].concat()
    //hash_password(password_begin, password_middle, password_end).await
}

pub async fn hash_password(password: String) -> String {
    let salt = SaltString::generate(&mut OsRng);
    warn!("Salt: {}", salt);

    // Hash password to PHC string ($pbkdf2-sha256$...)
    let password_hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    password_hash
}

pub async fn verify_hashed_password(password: String, hashed_password: String) -> bool {
    let parsed_hash = PasswordHash::new(&hashed_password).unwrap();
    Pbkdf2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub async fn generate_session_token(id: String) -> String {
    // Generate a unique session token
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let random_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    format!("{}:{}:{}", id, timestamp, random_string)
}

// 1 hour expiration time
pub async fn validate_token_expiration(token: String) -> bool {
    let parts: Vec<&str> = token.split(':').collect();
    let timestamp = parts[1].parse::<u64>().unwrap();
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let elapsed_time = current_timestamp - timestamp;
    elapsed_time < 3600
}

#[cfg(test)]
mod tests {
    use tokio::runtime::Runtime;

    use super::*;

    #[test]
    fn test_generate_handle() {
        let rt = Runtime::new().unwrap();
        let handle = rt.block_on(generate_handle("John".to_string(), "Doe".to_string()));
        assert_eq!(handle, "jdoe");
    }

    #[test]
    fn test_generate_random_password() {
        let rt = Runtime::new().unwrap();
        let password = rt.block_on(generate_random_password());
        assert_eq!(password.len(), 9);
    }

    #[test]
    fn test_hash_password() {
        let rt = Runtime::new().unwrap();
        let password = "password123".to_string();
        let hashed_password = rt.block_on(hash_password(password.clone()));
        assert!(hashed_password.starts_with("$pbkdf2-sha256$"));
    }

    #[test]
    fn test_verify_hashed_password() {
        let rt = Runtime::new().unwrap();
        let password = "password123".to_string();
        let hashed_password = rt.block_on(hash_password(password.clone()));
        let is_valid = rt.block_on(verify_hashed_password(password, hashed_password));
        assert!(is_valid);
    }

    #[test]
    fn test_generate_session_token() {
        let rt = Runtime::new().unwrap();
        let token = rt.block_on(generate_session_token("user123".to_string()));
        let parts: Vec<&str> = token.split(':').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "user123");
    }

    #[test]
    fn test_validate_token_expiration() {
        let rt = Runtime::new().unwrap();
        let token = rt.block_on(generate_session_token("user123".to_string()));
        let is_valid = rt.block_on(validate_token_expiration(token));
        assert!(is_valid);
    }

    #[test]
    fn test_expired_token() {
        let rt = Runtime::new().unwrap();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 3601;
        let token = format!("user123:{}:randomstring", timestamp);
        let is_valid = rt.block_on(validate_token_expiration(token));
        assert!(!is_valid);
    }
}

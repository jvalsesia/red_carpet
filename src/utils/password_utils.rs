use log::warn;
use pbkdf2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use rand::seq::SliceRandom;

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

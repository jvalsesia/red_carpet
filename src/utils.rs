use rand::seq::SliceRandom;

pub const LOWER_CASE: &str = "abcdefghijklmnopqrstuvxyz";
pub const UPPER_CASE: &str = "ABCDEFGHIJKLMNOPQRSTUVXYZ";
pub const SPECIAL_CHARACTER: &str = "!@#$%&*()_-+=,.:;?/|";
pub const NUMBERS: &str = "1234567890";
pub async fn generate_handle(first_name: String, last_name: String) -> String {
    format!(
        "{}{}",
        first_name.to_lowercase().chars().next().unwrap(),
        last_name.to_lowercase()
    )
}

pub async fn generate_random_password() -> String {
    let begin = [LOWER_CASE, SPECIAL_CHARACTER, NUMBERS].concat();
    let b: Vec<char> = begin.chars().collect();
    let password_begin: String = (0..1)
        .map(|_| *b.choose(&mut rand::thread_rng()).unwrap())
        .collect();

    let middle = [LOWER_CASE, UPPER_CASE, NUMBERS, SPECIAL_CHARACTER].concat();
    let m: Vec<char> = middle.chars().collect();
    let password_middle: String = (0..7)
        .map(|_| *m.choose(&mut rand::thread_rng()).unwrap())
        .collect();

    let end = [LOWER_CASE, UPPER_CASE, NUMBERS].concat();
    let e: Vec<char> = end.chars().collect();
    let password_end: String = (0..1)
        .map(|_| *e.choose(&mut rand::thread_rng()).unwrap())
        .collect();
    [password_begin, password_middle, password_end].concat()
}

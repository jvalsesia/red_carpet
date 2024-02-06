use rand::seq::SliceRandom;

pub async fn generate_handle(first_name: String, last_name: String) -> String {
    format!(
        "{}{}",
        first_name.to_lowercase().chars().nth(0).unwrap(),
        last_name.to_ascii_lowercase()
    )
}

pub async fn generate_random_password() -> String {
    let lower_case = "abcdefghijklmnopqrstuvxyz";
    let upper_case = "ABCDEFGHIJKLMNOPQRSTUVXYZ";
    let numbers = "1234567890";
    let signs = "!@#$%&*()_-+=,.:;?/|";

    let begin = [lower_case, signs, numbers].concat();
    let b: Vec<char> = begin.chars().collect();
    let password_begin: String = (0..1)
        .map(|_| *b.choose(&mut rand::thread_rng()).unwrap())
        .collect();

    let middle = [lower_case, upper_case, numbers, signs].concat();
    let m: Vec<char> = middle.chars().collect();
    let password_middle: String = (0..7)
        .map(|_| *m.choose(&mut rand::thread_rng()).unwrap())
        .collect();

    let end = [lower_case, upper_case, numbers].concat();
    let e: Vec<char> = end.chars().collect();
    let password_end: String = (0..1)
        .map(|_| *e.choose(&mut rand::thread_rng()).unwrap())
        .collect();
    [password_begin, password_middle, password_end].concat()
}

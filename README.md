# red_carpet

Red Carpet onboarding process - Rust

## Requirements

### User stories

As a new employee, I want to submit my personal details, so that the Avaya IT department can create a user handle and a password.

As an IT technician, I want to see the list of user ids that must be onboarded, so that I can see what Ids must be processed today.

As an IT technician, I want the solution to automatically generate a user handle and a random password for new employees, so that I can have time to learn Rust.

As a new employee, I want to securely view my submission details, so that I can check if there are any errors.

### Functional requirements

- Standalone web application with these REST API endpoints:

- [POST] Create an onboarding request by future employee. Validate that the person is at least 18 years old and has at least one diploma.

- [LIST] All user Ids that have not yet been onboarded.

- [PATCH] Update by Avaya IT. Set the employee’s global handle and generated password.

- [GET] Secured endpoint (basic auth) to view the employee details after IT set up the password. Generated passwords must be unique, exactly 9 characters long, include: an uppercase character, a number and a special character. The special character cannot be at the end, and the uppercase character not at the start. Therefore Avaya123! Is not a valid password.

- User handles must be unique and must be usable in an email address. They should have a meaningful relation to the employee’s first and last name.

### Non-functional requirements

- The data should be persisted
- It should support concurrent access

### Out of scope

- Transport of the credentials to the employee; assume there is a secure way to pass these from IT to the employee
- High performance

## Init

```sh
cargo init
```

## Dependencies

```sh
cargo add axum -F macros
cargo add axum-auth
cargo add tokio -F full
cargo add serde -F derive
cargo add serde_json
cargo add pretty_env_logger
cargo add log
cargo add uuid -F v4,serde
cargo add thiserror
cargo add pbkdf2 -F simple
cargo add rand_core -F std
cargo add chrono -F serde
cargo add tera
cargo add rand
```

## Environment

Linux bash

```bash
echo 'export RUST_LOG=debug' >> ~/.bash_profile
source ~/.bash_profile
env
```

MacOS zsh

```zsh
echo 'export RUST_LOG=debug' >>  ~/.zshenv
source  ~/.zshenv
env
```

## References

1. <https://www.shuttle.rs/blog/2022/08/11/authentication-tutorial>
2. <https://docs.rs/axum/latest/axum/>
3. <https://docs.rs/tokio/latest/tokio/sync/index.html>
4. <https://docs.rs/rand/latest/rand/seq/trait.SliceRandom.html>
5. <https://codevoweb.com/create-a-simple-api-in-rust-using-the-axum-framework>

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash_password(password: &str) -> String {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Error on `hash_password`")
        .to_string();
    hashed_password
}

pub fn verify_password(hashed_password: &str, password: &str) -> bool {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hashed_password).expect("Error parsing hashed password");
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

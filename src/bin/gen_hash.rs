use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

fn main() {
    let passwords = [("admin", "admin"), ("password", "password")];
    for (label, pw) in &passwords {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(pw.as_bytes(), &salt)
            .unwrap()
            .to_string();
        println!("{}: {}", label, hash);
    }
}

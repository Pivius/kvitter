use argon2::{
	password_hash::{
		rand_core::OsRng, SaltString, PasswordHasher,
		PasswordHash, PasswordVerifier as _, Error
	},
	Argon2
};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
	let salt = SaltString::generate(&mut OsRng);
	let argon2 = Argon2::default();

	let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
	Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, Error> {
	let parsed_hash = PasswordHash::new(hash)?;
	Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_hash_and_verify_password() {
		let password = "my_secure_password";
		let hash = hash_password(password).unwrap();
		
		assert!(verify_password(password, &hash).unwrap(), "Password verification failed");
		assert!(!verify_password("wrong_password", &hash).unwrap(), "Wrong password should not verify");
	}

	#[test]
	fn test_hash_format() {
		let password = "another_secure_password";
		let hash = hash_password(password).unwrap();

		assert!(!hash.starts_with("$argon2i$"), "Hash should not start with $argon2i$");
	}
}
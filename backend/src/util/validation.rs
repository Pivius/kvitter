use crate::util::error::{AppError, AppResult};

pub fn validate_password(password: &str) -> AppResult<()> {
	let validators: Vec<fn(&str) -> AppResult<()>> = vec![
		validate_password_length,
		validate_password_uppercase,
		validate_password_lowercase,
		validate_password_digit,
	];

	validators.iter()
		.try_fold((), |_, validator| validator(password))
}

fn validate_password_length(password: &str) -> AppResult<()> {
	match password.len() {
		0 => Err(AppError::BadRequest("Password cannot be empty".into())),
		len if len < 8 => 
			Err(AppError::BadRequest("Password must be at least 8 characters long".into())
		),
		len if len > 128 => Err(AppError::BadRequest("Password is too long".into())),
		_ => Ok(()),
	}
}

fn validate_password_uppercase(password: &str) -> AppResult<()> {
	match password.chars().any(|c| c.is_uppercase()) {
		true => Ok(()),
		false => Err(
			AppError::BadRequest("Password must contain at least one uppercase letter".into())
		),
	}
}

fn validate_password_lowercase(password: &str) -> AppResult<()> {
	match password.chars().any(|c| c.is_lowercase()) {
		true => Ok(()),
		false => Err(
			AppError::BadRequest("Password must contain at least one lowercase letter".into())
		),
	}
}

fn validate_password_digit(password: &str) -> AppResult<()> {
	match password.chars().any(|c| c.is_digit(10)) {
		true => Ok(()),
		false => Err(
			AppError::BadRequest("Password must contain at least one digit".into())
		),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_validate_password() {
		assert!(validate_password("Valid1").is_err());
		assert!(validate_password("validpassword").is_err());
		assert!(validate_password("VALIDPASSWORD").is_err());
		assert!(validate_password("ValidPassword").is_err());
		assert!(validate_password("Valid1Password").is_ok());
		assert!(validate_password("Valid1Password!").is_ok());
	}
}

use uuid::Uuid;

use crate::errors::AppError;

#[must_use]
pub fn validate_environment(environment: &str) -> bool {
    ["development", "staging", "production"].contains(&environment)
}

pub fn validate_deployment(environment: &str, contract_ids: &[Uuid]) -> Result<(), AppError> {
    if environment.is_empty() {
        return Err(AppError::validation("environment must not be empty"));
    }

    let valid_environments = ["development", "staging", "production"];
    if !valid_environments.contains(&environment) {
        return Err(AppError::validation(format!(
            "invalid environment '{environment}': must be one of {valid}",
            valid = valid_environments.join(", "),
        )));
    }

    if contract_ids.is_empty() {
        return Err(AppError::validation("at least one contract must be specified for deployment"));
    }

    Ok(())
}

pub const VALID_ENVIRONMENTS: &[&str] = &["development", "staging", "production"];

/// Validates that a string is a recognized deployment environment name.
#[must_use]
pub fn validate_environment(name: &str) -> bool {
    VALID_ENVIRONMENTS.contains(&name)
}

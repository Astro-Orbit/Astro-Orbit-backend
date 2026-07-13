use std::collections::HashSet;

use crate::errors::AppError;
use crate::permissions::role::Role;

/// Evaluates whether a given role has the required permission.
///
/// Supports both single permission checks and combined checks.
#[derive(Debug, Clone)]
pub struct PolicyEngine;

impl PolicyEngine {
    /// Checks that the role has the specified permission.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Forbidden` if the role lacks the permission.
    pub fn check(role: &Role, required: &str) -> Result<(), AppError> {
        if role.has_permission(required) {
            Ok(())
        } else {
            Err(AppError::forbidden(format!("missing required permission: {required}")))
        }
    }

    /// Checks that the role has ALL specified permissions.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Forbidden` listing all missing permissions.
    pub fn check_all(role: &Role, required: &[&str]) -> Result<(), AppError> {
        let missing: Vec<&str> = required.iter().filter(|perm| !role.has_permission(perm)).copied().collect();

        if missing.is_empty() {
            Ok(())
        } else {
            Err(AppError::forbidden(format!("missing required permissions: {}", missing.join(", "))))
        }
    }

    /// Checks that the role has ANY of the specified permissions.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Forbidden` if the role lacks all listed permissions.
    pub fn check_any(role: &Role, required: &[&str]) -> Result<(), AppError> {
        if required.iter().any(|perm| role.has_permission(perm)) {
            Ok(())
        } else {
            Err(AppError::forbidden(format!("missing any of required permissions: {}", required.join(", "))))
        }
    }

    /// Returns the intersection of role permissions and required permissions.
    #[must_use]
    pub fn effective_permissions(role: &Role, scopes: &HashSet<String>) -> HashSet<String> {
        let role_perms = role.permissions();
        role_perms.iter().filter(|p| scopes.contains(&p.to_string())).map(|p| (*p).to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_success() {
        let result = PolicyEngine::check(&Role::Admin, "org:update");
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_forbidden() {
        let result = PolicyEngine::check(&Role::Viewer, "project:create");
        assert!(result.is_err());
    }

    #[test]
    fn test_check_all_success() {
        let result = PolicyEngine::check_all(&Role::Owner, &["org:delete", "org:transfer"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_all_failure() {
        let result = PolicyEngine::check_all(&Role::Developer, &["org:delete", "org:update"]);
        assert!(result.is_err());
    }
}

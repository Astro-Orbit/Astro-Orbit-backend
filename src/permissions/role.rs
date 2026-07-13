use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::errors::AppError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Owner,
    Admin,
    Developer,
    #[default]
    Viewer,
}

impl Role {
    /// Returns the set of permissions granted to this role.
    #[must_use]
    pub fn permissions(&self) -> HashSet<&'static str> {
        match self {
            Self::Owner => Self::owner_permissions(),
            Self::Admin => Self::admin_permissions(),
            Self::Developer => Self::developer_permissions(),
            Self::Viewer => Self::viewer_permissions(),
        }
    }

    fn owner_permissions() -> HashSet<&'static str> {
        Self::admin_permissions()
            .union(&HashSet::from(["org:delete", "org:transfer", "org:members:remove_owner"]))
            .copied()
            .collect()
    }

    fn admin_permissions() -> HashSet<&'static str> {
        Self::developer_permissions()
            .union(&HashSet::from([
                "org:update",
                "org:members:invite",
                "org:members:remove",
                "org:members:update_role",
                "api_key:create",
                "api_key:delete",
                "audit:read",
                "billing:read",
                "billing:update",
            ]))
            .copied()
            .collect()
    }

    fn developer_permissions() -> HashSet<&'static str> {
        Self::viewer_permissions()
            .union(&HashSet::from([
                "project:create",
                "project:update",
                "project:delete",
                "contract:create",
                "contract:update",
                "contract:delete",
                "contract:deploy",
                "deployment:create",
                "deployment:cancel",
                "deployment:rollback",
            ]))
            .copied()
            .collect()
    }

    fn viewer_permissions() -> HashSet<&'static str> {
        HashSet::from([
            "org:read",
            "project:read",
            "contract:read",
            "deployment:read",
            "analytics:read",
            "api_key:read",
            "notification:read",
        ])
    }

    /// Returns true if this role has the specified permission.
    #[must_use]
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions().contains(permission)
    }
}

impl FromStr for Role {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "developer" | "dev" => Ok(Self::Developer),
            "viewer" => Ok(Self::Viewer),
            other => Err(AppError::bad_request(format!("invalid role: {other}"))),
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Owner => write!(f, "owner"),
            Self::Admin => write!(f, "admin"),
            Self::Developer => write!(f, "developer"),
            Self::Viewer => write!(f, "viewer"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owner_has_all_permissions() {
        let role = Role::Owner;
        assert!(role.has_permission("org:delete"));
        assert!(role.has_permission("org:transfer"));
        assert!(role.has_permission("project:create"));
        assert!(role.has_permission("contract:read"));
    }

    #[test]
    fn test_viewer_has_read_only() {
        let role = Role::Viewer;
        assert!(role.has_permission("org:read"));
        assert!(role.has_permission("project:read"));
        assert!(!role.has_permission("project:create"));
        assert!(!role.has_permission("org:update"));
    }

    #[test]
    fn test_admin_has_org_permissions() {
        let role = Role::Admin;
        assert!(role.has_permission("org:update"));
        assert!(role.has_permission("org:members:invite"));
        assert!(role.has_permission("project:create"));
        assert!(!role.has_permission("org:delete"));
    }

    #[test]
    fn test_role_parse() {
        assert_eq!("owner".parse::<Role>().unwrap(), Role::Owner);
        assert_eq!("admin".parse::<Role>().unwrap(), Role::Admin);
        assert_eq!("developer".parse::<Role>().unwrap(), Role::Developer);
        assert_eq!("dev".parse::<Role>().unwrap(), Role::Developer);
        assert_eq!("viewer".parse::<Role>().unwrap(), Role::Viewer);
    }

    #[test]
    fn test_permission_matrix_coverage() {
        let all_permissions = [
            "org:read",
            "org:update",
            "org:delete",
            "org:transfer",
            "org:members:invite",
            "org:members:remove",
            "org:members:update_role",
            "org:members:remove_owner",
            "project:read",
            "project:create",
            "project:update",
            "project:delete",
            "contract:read",
            "contract:create",
            "contract:update",
            "contract:delete",
            "contract:deploy",
            "deployment:read",
            "deployment:create",
            "deployment:cancel",
            "deployment:rollback",
            "api_key:read",
            "api_key:create",
            "api_key:delete",
            "audit:read",
            "analytics:read",
            "notification:read",
            "billing:read",
            "billing:update",
        ];
        for perm in &all_permissions {
            let has_owner = Role::Owner.has_permission(perm);
            let _has_admin = Role::Admin.has_permission(perm);
            let has_dev = Role::Developer.has_permission(perm);
            let has_viewer = Role::Viewer.has_permission(perm);
            assert!(has_owner, "Owner should have {perm}");

            let _ = (has_dev, has_viewer);
        }
    }
}

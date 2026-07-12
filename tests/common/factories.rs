//! Test data factories.
//!
//! Builder-pattern factories for creating test entities.
//! Each factory produces a struct ready to be inserted via the
//! corresponding repository.

use uuid::Uuid;

pub struct UserFactory {
    pub stellar_public: String,
    pub display_name: Option<String>,
}

impl UserFactory {
    pub fn new() -> Self {
        Self {
            stellar_public: format!("G{}", Uuid::new_v4().to_string().replace('-', "").to_uppercase()),
            display_name: None,
        }
    }

    pub fn with_display_name(mut self, name: &str) -> Self {
        self.display_name = Some(name.to_string());
        self
    }
}

pub struct OrgFactory {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

impl OrgFactory {
    pub fn new() -> Self {
        let slug = Uuid::new_v4().to_string()[..8].to_string();
        Self {
            name: format!("Org {slug}"),
            slug,
            description: None,
        }
    }
}

pub struct ProjectFactory {
    pub name: String,
    pub slug: String,
}

impl ProjectFactory {
    pub fn new() -> Self {
        let slug = Uuid::new_v4().to_string()[..8].to_string();
        Self {
            name: format!("Project {slug}"),
            slug,
        }
    }
}

/// Validates that a string is a valid URL slug.
///
/// Slugs must be 1-100 characters, lowercase alphanumeric with hyphens.
#[must_use]
pub fn validate_slug(slug: &str) -> bool {
    if slug.is_empty() || slug.len() > 100 {
        return false;
    }
    slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

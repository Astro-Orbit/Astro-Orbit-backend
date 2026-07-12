use async_trait::async_trait;
use uuid::Uuid;

use crate::models::org::Organization;

/// Organization repository trait.
#[async_trait]
pub trait OrgRepository: Send + Sync {
    async fn create(&self, name: &str, slug: &str, description: Option<&str>) -> Result<Organization, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Organization, sqlx::Error>;
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Organization>, sqlx::Error>;
    async fn update(&self, id: Uuid, name: Option<&str>, description: Option<&str>) -> Result<Organization, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error>;
    async fn add_member(&self, org_id: Uuid, user_id: Uuid, role: &str) -> Result<(), sqlx::Error>;
    async fn remove_member(&self, org_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error>;
    async fn update_member_role(&self, org_id: Uuid, user_id: Uuid, role: &str) -> Result<(), sqlx::Error>;
}

pub struct PgOrgRepository {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl PgOrgRepository {
    #[must_use]
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrgRepository for PgOrgRepository {
    async fn create(&self, name: &str, slug: &str, description: Option<&str>) -> Result<Organization, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            r"
            INSERT INTO organizations (name, slug, description)
            VALUES ($1, $2, $3)
            RETURNING id, name, slug, description, avatar_url, plan, settings,
                      created_at, updated_at, deleted_at
            ",
        )
        .bind(name)
        .bind(slug)
        .bind(description)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Organization, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            r"
            SELECT id, name, slug, description, avatar_url, plan, settings,
                   created_at, updated_at, deleted_at
            FROM organizations
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Organization>, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            r"
            SELECT o.id, o.name, o.slug, o.description, o.avatar_url, o.plan, o.settings,
                   o.created_at, o.updated_at, o.deleted_at
            FROM organizations o
            JOIN organization_members om ON om.organization_id = o.id
            WHERE om.user_id = $1 AND o.deleted_at IS NULL
            ",
        )
        .bind(user_id)
        .fetch_all(&*self.pool)
        .await
    }

    async fn update(&self, id: Uuid, name: Option<&str>, description: Option<&str>) -> Result<Organization, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            r"
            UPDATE organizations
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, name, slug, description, avatar_url, plan, settings,
                      created_at, updated_at, deleted_at
            ",
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .fetch_one(&*self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r"UPDATE organizations SET deleted_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn add_member(&self, org_id: Uuid, user_id: Uuid, role: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r"
            INSERT INTO organization_members (organization_id, user_id, role)
            VALUES ($1, $2, $3)
            ",
        )
        .bind(org_id)
        .bind(user_id)
        .bind(role)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn remove_member(&self, org_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r"
            DELETE FROM organization_members
            WHERE organization_id = $1 AND user_id = $2
            ",
        )
        .bind(org_id)
        .bind(user_id)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn update_member_role(&self, org_id: Uuid, user_id: Uuid, role: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r"
            UPDATE organization_members
            SET role = $3
            WHERE organization_id = $1 AND user_id = $2
            ",
        )
        .bind(org_id)
        .bind(user_id)
        .bind(role)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}

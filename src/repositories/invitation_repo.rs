use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::invitation::OrganizationInvitation;

#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait InvitationRepository: Send + Sync {
    async fn create(
        &self,
        org_id: Uuid,
        invited_by: Uuid,
        email: Option<&str>,
        user_id: Option<Uuid>,
        token_hash: &str,
        role: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<OrganizationInvitation, AppError>;
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<OrganizationInvitation, AppError>;
    async fn find_by_org(&self, org_id: Uuid) -> Result<Vec<OrganizationInvitation>, AppError>;
    async fn update_status(&self, id: Uuid, status: &str) -> Result<(), AppError>;
    async fn find_pending_by_email(&self, email: &str) -> Result<Vec<OrganizationInvitation>, AppError>;
    async fn find_pending_by_user(&self, user_id: Uuid) -> Result<Vec<OrganizationInvitation>, AppError>;
}

pub struct PgInvitationRepository {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl PgInvitationRepository {
    #[must_use]
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InvitationRepository for PgInvitationRepository {
    async fn create(
        &self,
        org_id: Uuid,
        invited_by: Uuid,
        email: Option<&str>,
        user_id: Option<Uuid>,
        token_hash: &str,
        role: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<OrganizationInvitation, AppError> {
        sqlx::query_as::<_, OrganizationInvitation>(
            r"
            INSERT INTO organization_invitations (organization_id, invited_by, invitee_email, invitee_user_id, token_hash, role, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, organization_id, invited_by, invitee_email, invitee_user_id,
                      token_hash, role, status, expires_at, created_at, updated_at
            ",
        )
        .bind(org_id)
        .bind(invited_by)
        .bind(email)
        .bind(user_id)
        .bind(token_hash)
        .bind(role)
        .bind(expires_at)
        .fetch_one(&*self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<OrganizationInvitation, AppError> {
        sqlx::query_as::<_, OrganizationInvitation>(
            r"
            SELECT id, organization_id, invited_by, invitee_email, invitee_user_id,
                   token_hash, role, status, expires_at, created_at, updated_at
            FROM organization_invitations
            WHERE token_hash = $1
            ",
        )
        .bind(token_hash)
        .fetch_one(&*self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn find_by_org(&self, org_id: Uuid) -> Result<Vec<OrganizationInvitation>, AppError> {
        sqlx::query_as::<_, OrganizationInvitation>(
            r"
            SELECT id, organization_id, invited_by, invitee_email, invitee_user_id,
                   token_hash, role, status, expires_at, created_at, updated_at
            FROM organization_invitations
            WHERE organization_id = $1
            ORDER BY created_at DESC
            ",
        )
        .bind(org_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn update_status(&self, id: Uuid, status: &str) -> Result<(), AppError> {
        sqlx::query(
            r"
            UPDATE organization_invitations
            SET status = $2, updated_at = NOW()
            WHERE id = $1
            ",
        )
        .bind(id)
        .bind(status)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn find_pending_by_email(&self, email: &str) -> Result<Vec<OrganizationInvitation>, AppError> {
        sqlx::query_as::<_, OrganizationInvitation>(
            r"
            SELECT id, organization_id, invited_by, invitee_email, invitee_user_id,
                   token_hash, role, status, expires_at, created_at, updated_at
            FROM organization_invitations
            WHERE invitee_email = $1 AND status = 'pending'
            ORDER BY created_at DESC
            ",
        )
        .bind(email)
        .fetch_all(&*self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn find_pending_by_user(&self, user_id: Uuid) -> Result<Vec<OrganizationInvitation>, AppError> {
        sqlx::query_as::<_, OrganizationInvitation>(
            r"
            SELECT id, organization_id, invited_by, invitee_email, invitee_user_id,
                   token_hash, role, status, expires_at, created_at, updated_at
            FROM organization_invitations
            WHERE invitee_user_id = $1 AND status = 'pending'
            ORDER BY created_at DESC
            ",
        )
        .bind(user_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(AppError::from)
    }
}

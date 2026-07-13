use std::sync::Arc;

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::errors::AppError;
use crate::permissions::role::Role;
use crate::repositories::invitation_repo::InvitationRepository;
use crate::repositories::org_repo::OrgRepository;
use crate::repositories::user_repo::UserRepository;
use crate::utils::crypto::random_hex;

pub struct OrgService {
    pool: Arc<sqlx::PgPool>,
    org_repo: Arc<dyn OrgRepository>,
    user_repo: Arc<dyn UserRepository>,
    invitation_repo: Arc<dyn InvitationRepository>,
}

impl OrgService {
    pub fn new(
        pool: Arc<sqlx::PgPool>,
        org_repo: Arc<dyn OrgRepository>,
        user_repo: Arc<dyn UserRepository>,
        invitation_repo: Arc<dyn InvitationRepository>,
    ) -> Self {
        Self { pool, org_repo, user_repo, invitation_repo }
    }

    pub async fn create(
        &self,
        name: &str,
        slug: &str,
        description: Option<&str>,
        owner_id: Uuid,
    ) -> Result<OrgResult, AppError> {
        let org = self.org_repo.create(name, slug, description).await?;
        self.org_repo.add_member(org.id, owner_id, "owner").await?;
        Ok(OrgResult {
            id: org.id,
            name: org.name,
            slug: org.slug,
            description: org.description,
            role: "owner".to_string(),
            member_count: 1,
            created_at: org.created_at,
        })
    }

    pub async fn get(&self, org_id: Uuid, user_id: Uuid) -> Result<OrgResult, AppError> {
        let org = self.org_repo.find_by_id(org_id).await?;
        let role = self.get_member_role(org_id, user_id).await?;
        let member_count = self.member_count(org_id).await?;
        Ok(OrgResult {
            id: org.id,
            name: org.name,
            slug: org.slug,
            description: org.description,
            role,
            member_count,
            created_at: org.created_at,
        })
    }

    pub async fn list(&self, user_id: Uuid) -> Result<Vec<OrgResult>, AppError> {
        let orgs = self.org_repo.find_by_user(user_id).await?;
        let mut results = Vec::new();
        for org in orgs {
            let role = self.get_member_role(org.id, user_id).await?;
            let member_count = self.member_count(org.id).await?;
            results.push(OrgResult {
                id: org.id,
                name: org.name,
                slug: org.slug,
                description: org.description,
                role,
                member_count,
                created_at: org.created_at,
            });
        }
        Ok(results)
    }

    pub async fn update(
        &self,
        org_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<OrgResult, AppError> {
        let org = self.org_repo.update(org_id, name, description).await?;
        Ok(OrgResult {
            id: org.id,
            name: org.name,
            slug: org.slug,
            description: org.description,
            role: String::new(),
            member_count: 0,
            created_at: org.created_at,
        })
    }

    pub async fn delete(&self, org_id: Uuid) -> Result<(), AppError> {
        self.org_repo.delete(org_id).await?;
        Ok(())
    }

    pub async fn invite(
        &self,
        org_id: Uuid,
        invited_by: Uuid,
        email: &str,
        role: &str,
    ) -> Result<InviteResult, AppError> {
        role.parse::<Role>()?;

        let token = random_hex::<32>();
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

        let invitee_user = self.user_repo.find_by_email(email).await.ok();

        let invitation = self
            .invitation_repo
            .create(org_id, invited_by, Some(email), invitee_user.map(|u| u.id), &token_hash, role, expires_at)
            .await?;

        Ok(InviteResult {
            id: invitation.id,
            token,
            organization_id: org_id,
            email: email.to_string(),
            role: role.to_string(),
            expires_at,
            created_at: invitation.created_at,
        })
    }

    pub async fn accept_invite(&self, token: &str, user_id: Uuid) -> Result<OrgResult, AppError> {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        let invitation = self.invitation_repo.find_by_token_hash(&token_hash).await?;

        if invitation.status != "pending" {
            return Err(AppError::bad_request("invitation already processed"));
        }
        if chrono::Utc::now() > invitation.expires_at {
            self.invitation_repo.update_status(invitation.id, "expired").await?;
            return Err(AppError::bad_request("invitation expired"));
        }

        self.org_repo.add_member(invitation.organization_id, user_id, &invitation.role).await?;
        self.invitation_repo.update_status(invitation.id, "accepted").await?;

        self.get(invitation.organization_id, user_id).await
    }

    pub async fn reject_invite(&self, token: &str) -> Result<(), AppError> {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        let invitation = self.invitation_repo.find_by_token_hash(&token_hash).await?;

        if invitation.status != "pending" {
            return Err(AppError::bad_request("invitation already processed"));
        }

        self.invitation_repo.update_status(invitation.id, "rejected").await?;
        Ok(())
    }

    async fn get_member_role(&self, org_id: Uuid, user_id: Uuid) -> Result<String, AppError> {
        let row = sqlx::query_as::<_, (String,)>(
            r"SELECT role FROM organization_members WHERE organization_id = $1 AND user_id = $2",
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(AppError::from)?;

        row.map(|r| r.0).ok_or_else(|| AppError::forbidden("not a member of this organization"))
    }

    async fn member_count(&self, org_id: Uuid) -> Result<i64, AppError> {
        let row = sqlx::query_as::<_, (i64,)>(r"SELECT COUNT(*) FROM organization_members WHERE organization_id = $1")
            .bind(org_id)
            .fetch_one(&*self.pool)
            .await
            .map_err(AppError::from)?;

        Ok(row.0)
    }
}

pub struct OrgResult {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub role: String,
    pub member_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct InviteResult {
    pub id: Uuid,
    pub token: String,
    pub organization_id: Uuid,
    pub email: String,
    pub role: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

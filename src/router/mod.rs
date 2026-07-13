use std::sync::Arc;

use axum::routing::{delete, get, patch, post};
use axum::Router;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::handlers;
use crate::middleware as midware;
use crate::state::AppState;

/// Builds the root application router with all routes and middleware.
pub fn build_router(config: &Config, state: &Arc<RwLock<AppState>>) -> Router {
    let ext = axum::Extension(state.clone());

    // --- Public routes (no auth required) ---
    let public_routes = Router::new()
        .route("/auth/challenge", post(handlers::auth::challenge))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh));

    // --- Protected routes (auth required) ---
    let protected_routes = Router::new()
        // Auth
        .route("/auth/logout", post(handlers::auth::logout))
        .route("/auth/logout-all", post(handlers::auth::logout_all))
        // Users
        .route("/me", get(handlers::users::get_me))
        .route("/me", patch(handlers::users::update_me))
        // Organizations
        .route("/organizations", post(handlers::orgs::create))
        .route("/organizations", get(handlers::orgs::list))
        .route("/organizations/:id", get(handlers::orgs::get_by_id))
        .route("/organizations/:id", patch(handlers::orgs::update))
        .route("/organizations/:id", delete(handlers::orgs::delete))
        .route("/organizations/:id/invite", post(handlers::orgs::invite))
        .route("/invitations/:token/accept", post(handlers::orgs::accept_invite))
        .route("/invitations/:token/reject", post(handlers::orgs::reject_invite))
        // API Keys
        .route("/api-keys", post(handlers::api_keys::create))
        .route("/api-keys", get(handlers::api_keys::list))
        .route("/api-keys/:id", delete(handlers::api_keys::delete))
        // Legacy routes (existing, kept for compatibility)
        .route("/orgs", post(handlers::orgs::create))
        .route("/orgs", get(handlers::orgs::list))
        .route("/orgs/:id", get(handlers::orgs::get_by_id))
        .route("/orgs/:id", patch(handlers::orgs::update))
        .route("/orgs/:id", delete(handlers::orgs::delete))
        .route("/orgs/:id/members", post(handlers::orgs::add_member))
        .route("/orgs/:id/members", get(handlers::orgs::list_members))
        .route("/orgs/:id/members/:user_id", patch(handlers::orgs::update_member))
        .route("/orgs/:id/members/:user_id", delete(handlers::orgs::remove_member))
        .route("/orgs/:id/roles", get(handlers::orgs::list_roles))
        .route("/orgs/:id/roles", post(handlers::orgs::create_role))
        // Projects
        .route("/orgs/:id/projects", post(handlers::projects::create))
        .route("/orgs/:id/projects", get(handlers::projects::list))
        .route("/projects/:id", get(handlers::projects::get_by_id))
        .route("/projects/:id", patch(handlers::projects::update))
        .route("/projects/:id", delete(handlers::projects::delete))
        // Contracts
        .route("/projects/:id/contracts", post(handlers::contracts::create))
        .route("/projects/:id/contracts", get(handlers::contracts::list))
        .route("/contracts/:id", get(handlers::contracts::get_by_id))
        .route("/contracts/:id", patch(handlers::contracts::update))
        .route("/contracts/:id/versions", post(handlers::contracts::create_version))
        .route("/contracts/:id/versions", get(handlers::contracts::list_versions))
        // Deployments
        .route("/projects/:id/deployments", post(handlers::deployments::create))
        .route("/projects/:id/deployments", get(handlers::deployments::list))
        .route("/deployments/:id", get(handlers::deployments::get_by_id))
        .route("/deployments/:id/rollback", post(handlers::deployments::rollback))
        .route("/deployments/:id/cancel", post(handlers::deployments::cancel))
        .route("/deployments/:id/logs", get(handlers::deployments::logs))
        // Explorer
        .route("/explorer/contracts/:id", get(handlers::explorer::contract_details))
        .route("/explorer/transactions", get(handlers::explorer::transactions))
        .route("/explorer/transactions/:id", get(handlers::explorer::transaction_details))
        .route("/explorer/events", get(handlers::explorer::events))
        .route("/explorer/events/:id", get(handlers::explorer::event_details))
        // Wallets
        .route("/wallets", post(handlers::wallets::create))
        .route("/wallets", get(handlers::wallets::list))
        .route("/wallets/:id", get(handlers::wallets::get_by_id))
        .route("/wallets/:id", patch(handlers::wallets::update))
        .route("/wallets/:id", delete(handlers::wallets::delete))
        // API Keys (legacy)
        .route("/orgs/:id/api-keys", post(handlers::api_keys::create))
        .route("/orgs/:id/api-keys", get(handlers::api_keys::list))
        .route("/orgs/:id/api-keys/:key_id", delete(handlers::api_keys::delete))
        // Analytics
        .route("/orgs/:id/analytics/overview", get(handlers::analytics::overview))
        .route("/projects/:id/analytics/contract-calls", get(handlers::analytics::contract_calls))
        .route("/projects/:id/analytics/gas", get(handlers::analytics::gas_usage))
        .route("/projects/:id/analytics/users", get(handlers::analytics::active_users))
        // Notifications
        .route("/notifications", get(handlers::notifications::list))
        .route("/notifications/:id/read", patch(handlers::notifications::mark_read))
        .route("/notifications/preferences", post(handlers::notifications::preferences))
        // Security
        .route("/contracts/:id/scan", post(handlers::security::create_scan))
        .route("/contracts/:id/scans", get(handlers::security::list_scans))
        .route("/scans/:id", get(handlers::security::get_scan))
        .route("/scans/:id/findings", get(handlers::security::findings))
        // Users (legacy)
        .route("/users/me", get(handlers::users::get_me))
        .route("/users/me", patch(handlers::users::update_me))
        .route("/users/:id", get(handlers::users::get_by_id))
        // Admin
        .route("/admin/users", get(handlers::admin::users))
        .route("/admin/orgs", get(handlers::admin::orgs))
        .route("/admin/stats", get(handlers::admin::stats))
        // Auth middleware for all protected routes
        .route_layer(axum::middleware::from_fn(midware::auth::middleware));

    // --- Health & Observability (always public) ---
    let health_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/ready", get(handlers::health::ready_check))
        .route("/live", get(handlers::health::live_check))
        .route("/version", get(handlers::health::version));

    // --- Combine routes into v1 ---
    let v1_router = Router::new().merge(public_routes).merge(protected_routes).merge(health_routes).layer(ext);

    let app = Router::new().nest("/v1", v1_router);

    midware::apply(config, app)
}

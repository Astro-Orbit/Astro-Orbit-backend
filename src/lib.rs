#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::future_not_send)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::similar_names)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::default_trait_access)]

pub mod auth;
pub mod cache;
pub mod config;
pub mod database;
pub mod deployment;
pub mod dto;
pub mod errors;
pub mod events;
pub mod handlers;
pub mod jobs;
pub mod middleware;
pub mod models;
pub mod notifications;
pub mod permissions;
pub mod repositories;
pub mod response;
pub mod router;
pub mod services;
pub mod stellar;
pub mod telemetry;
pub mod utils;
pub mod validation;

pub use errors::AppError;
pub use response::ApiResponse;

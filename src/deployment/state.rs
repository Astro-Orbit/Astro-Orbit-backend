use std::str::FromStr;

use crate::errors::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeploymentState {
    Pending,
    Building,
    Scanning,
    Deploying,
    Deployed,
    Failed,
    Cancelled,
    RolledBack,
}

impl FromStr for DeploymentState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "building" => Ok(Self::Building),
            "scanning" => Ok(Self::Scanning),
            "deploying" => Ok(Self::Deploying),
            "deployed" => Ok(Self::Deployed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            "rolled_back" => Ok(Self::RolledBack),
            _ => Err(()),
        }
    }
}

impl DeploymentState {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Building => "building",
            Self::Scanning => "scanning",
            Self::Deploying => "deploying",
            Self::Deployed => "deployed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::RolledBack => "rolled_back",
        }
    }

    pub fn can_transition_to(&self, next: DeploymentState) -> Result<(), AppError> {
        match (*self, next) {
            (Self::Pending, Self::Building | Self::Cancelled)
            | (Self::Building, Self::Scanning | Self::Failed)
            | (Self::Scanning, Self::Deploying | Self::Failed)
            | (Self::Deploying, Self::Deployed | Self::Failed)
            | (Self::Deployed, Self::RolledBack) => Ok(()),
            (current, target) => Err(AppError::validation(format!(
                "cannot transition from {current} to {target}",
                current = current.as_str(),
                target = target.as_str(),
            ))),
        }
    }
}

impl std::fmt::Display for DeploymentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

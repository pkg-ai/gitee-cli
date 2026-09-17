use crate::config::ConfigStore;

pub const EXIT_OK: u8 = 0;
pub const EXIT_USAGE: u8 = 2;
pub const EXIT_AUTH: u8 = 3;
pub const EXIT_CONFIG: u8 = 4;
pub const EXIT_REMOTE: u8 = 5;
pub const EXIT_NOT_FOUND: u8 = 6;
pub const EXIT_GIT: u8 = 7;

pub struct CommandOutcome {
    pub code: u8,
    pub stdout: Option<String>,
}

impl CommandOutcome {
    pub fn json(code: u8, payload: serde_json::Value) -> Self {
        Self {
            code,
            stdout: Some(payload.to_string()),
        }
    }

    pub fn text(code: u8, body: String) -> Self {
        Self {
            code,
            stdout: Some(body),
        }
    }
}

pub struct CommandError {
    pub code: u8,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl CommandError {
    pub fn usage(message: impl Into<String>) -> Self {
        Self {
            code: EXIT_USAGE,
            stdout: None,
            stderr: Some(message.into()),
        }
    }

    pub fn config(error: impl std::fmt::Display) -> Self {
        Self {
            code: EXIT_CONFIG,
            stdout: None,
            stderr: Some(format!("config error: {error}")),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            code: EXIT_NOT_FOUND,
            stdout: None,
            stderr: Some(message.into()),
        }
    }

    pub fn git(message: impl Into<String>) -> Self {
        Self {
            code: EXIT_GIT,
            stdout: None,
            stderr: Some(message.into()),
        }
    }

    /// A remote API call failed due to bad/absent credentials.
    pub fn auth() -> Self {
        Self {
            code: EXIT_AUTH,
            stdout: None,
            stderr: Some("authentication failed".to_string()),
        }
    }

    /// A remote API call failed at the transport layer.
    pub fn remote_transport(error: impl std::fmt::Display) -> Self {
        Self {
            code: EXIT_REMOTE,
            stdout: None,
            stderr: Some(format!("remote request failed: {error}")),
        }
    }

    /// A remote API call returned an unexpected status.
    pub fn remote_status(status: u16) -> Self {
        Self {
            code: EXIT_REMOTE,
            stdout: None,
            stderr: Some(format!(
                "remote request returned unexpected status: {status}"
            )),
        }
    }

    /// A remote API call failed with an explicit status and message.
    pub fn remote_status_message(status: u16, message: String) -> Self {
        Self {
            code: EXIT_REMOTE,
            stdout: None,
            stderr: Some(format!("remote request failed ({status}): {message}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json { fields: Option<Vec<String>> },
}

impl OutputFormat {
    pub fn json_fields(&self) -> Option<&[String]> {
        match self {
            Self::Json {
                fields: Some(fields),
            } => Some(fields),
            Self::Text | Self::Json { fields: None } => None,
        }
    }
}

/// Shared token plumbing for services that carry a [`ConfigStore`]. Resolves an
/// optional runtime token (env first, then saved config) and can demand one for
/// mutating commands.
pub trait TokenRequester {
    fn config_store(&self) -> &ConfigStore;

    fn token(&self) -> Result<Option<String>, CommandError> {
        self.config_store()
            .load_runtime_token()
            .map_err(CommandError::config)
            .map(|resolved| resolved.map(|resolved| resolved.token))
    }

    fn require_token(&self, action: &str) -> Result<String, CommandError> {
        self.token()?.ok_or_else(|| CommandError {
            code: EXIT_AUTH,
            stdout: None,
            stderr: Some(format!("authentication required for {action}")),
        })
    }
}

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CmdError {
    pub message: String,
}

impl From<anyhow::Error> for CmdError {
    fn from(e: anyhow::Error) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

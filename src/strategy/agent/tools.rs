pub mod access_control;
pub mod account;
pub mod buy;
pub mod open_orders;
pub mod positions;
pub mod quotes;
pub mod sell;
pub mod web_fetch;

use std::fmt;

#[derive(Debug)]
pub struct ToolCallError(String);

impl fmt::Display for ToolCallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ToolCallError {}

impl From<crate::error::GreedError> for ToolCallError {
    fn from(e: crate::error::GreedError) -> Self {
        ToolCallError(e.to_string())
    }
}

impl From<reqwest::Error> for ToolCallError {
    fn from(e: reqwest::Error) -> Self {
        ToolCallError(e.to_string())
    }
}

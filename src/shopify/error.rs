use crate::graphql::GraphQLError;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum Error {
    GraphQL(Vec<GraphQLError>),
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
    Custom(String),
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(error)
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Custom(message) => message.fmt(f),
            _ => write!(f, "An unexpected error has occurred: `{:?}`", self),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

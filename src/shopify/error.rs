use crate::graphql::GraphQLError;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum Error {
    GraphQL(Vec<GraphQLError>),
    Reqwest(reqwest::Error),
    Json,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(error)
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

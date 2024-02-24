use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum GraphQLResponse<T> {
    Success { data: T },
    Error { errors: Vec<GraphQLError> },
}

#[derive(Deserialize, Debug)]
pub struct Connection<T> {
    pub nodes: Vec<T>,
}

#[derive(Deserialize, Debug)]
pub struct GraphQLError {
    pub message: Option<String>,
    pub locations: Option<Vec<Location>>,
    pub extensions: Value,
}

#[derive(Deserialize, Debug)]
pub struct Location {
    pub line: u32,
    pub column: u32,
}

use crate::graphql::Connection;
use serde::{Deserialize, Serialize};

mod client;
mod error;
mod order_number;

pub use client::Client;
pub use error::Error;
pub use order_number::OrderNumber;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub line_items: Connection<LineItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LineItem {
    pub sku: String,
    pub quantity: usize,
}

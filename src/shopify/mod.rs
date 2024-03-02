use crate::graphql::Connection;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;

mod client;
mod error;

pub use client::Client;
pub use error::Error;

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

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct OrderNumber(u32);

impl FromStr for OrderNumber {
    type Err = <u32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix('#') {
            Ok(Self(s.parse()?))
        } else {
            Ok(Self(s.parse()?))
        }
    }
}

impl Display for OrderNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

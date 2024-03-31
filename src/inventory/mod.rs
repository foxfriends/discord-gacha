use reqwest::Url;
use serde::Serialize;
use serde_json::json;

use crate::{error::CustomError, shopify::OrderNumber};

pub struct Client {
    url: Url,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct Item {
    sku: String,
    quantity: usize,
}

#[derive(Serialize)]
struct OrderData {
    items: Vec<Item>,
    data: serde_json::Value,
}

impl Client {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.parse().unwrap(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn log_pull(
        &self,
        discord_user_id: String,
        order_number: OrderNumber,
        sku: String,
    ) -> Result<(), crate::Error> {
        self.log_order(OrderData {
            data: json! {{
                "order_number": order_number,
                "discord_user_id": discord_user_id,
            }},
            items: vec![Item { sku, quantity: 1 }],
        }).await
    }

    async fn log_order(&self, order: OrderData) -> Result<(), crate::Error> {
        self.client
            .post(self.url.clone())
            .json(&order)
            .send()
            .await
            .map_err(|err| CustomError(format!("Failed to log order: {err}")))?;
        Ok(())
    }
}

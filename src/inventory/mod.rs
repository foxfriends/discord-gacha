use crate::shopify::OrderNumber;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

pub struct Client {
    url: Url,
    client: reqwest::Client,
    is_enabled: bool,
}

#[derive(Deserialize, Serialize)]
pub struct Item {
    sku: String,
    quantity: usize,
}

#[derive(Serialize)]
struct OrderData {
    source: &'static str,
    items: Vec<Item>,
    data: serde_json::Value,
}

impl Client {
    pub fn new(url: &str, is_enabled: bool) -> Self {
        log::warn!("Inventory logging active: {is_enabled}");
        Self {
            url: url.parse().unwrap(),
            client: reqwest::Client::new(),
            is_enabled,
        }
    }

    pub async fn log_pull(
        &self,
        discord_user_id: String,
        order_number: OrderNumber,
        sku: String,
    ) -> Result<(), reqwest::Error> {
        self.log_order(OrderData {
            source: "Discord Gacha",
            data: json! {{
                "order_number": order_number,
                "discord_user_id": discord_user_id,
            }},
            items: vec![Item { sku, quantity: 1 }],
        })
        .await
    }

    async fn log_order(&self, order: OrderData) -> Result<(), reqwest::Error> {
        if !self.is_enabled {
            return Ok(());
        }
        self.client
            .post(self.url.join("/custom/orders/create").unwrap())
            .json(&order)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn get_inventory(&self) -> Result<HashMap<String, usize>, reqwest::Error> {
        Ok(self
            .client
            .get(self.url.join("/google/view").unwrap())
            .send()
            .await?
            .json::<Vec<Item>>()
            .await?
            .into_iter()
            .map(|item| (item.sku, item.quantity))
            .collect())
    }
}

use reqwest::header::{HeaderMap, HeaderValue};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::json;

use super::{Error, Order, OrderNumber};
use crate::graphql::{Connection, GraphQLResponse};

pub struct Client {
    shop: String,
    client: reqwest::Client,
}

impl Client {
    pub fn new(shop: impl AsRef<str>, token: impl AsRef<str>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Shopify-Access-Token",
            HeaderValue::from_str(token.as_ref()).unwrap(),
        );
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        Self {
            shop: shop.as_ref().to_owned(),
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
        }
    }

    async fn post_graphql<T: DeserializeOwned>(&self, query: String) -> Result<T, Error> {
        let url = format!(
            "https://{}.myshopify.com/admin/api/2024-01/graphql.json",
            self.shop,
        );
        log::trace!("graphql query: {}", query);
        let response = self
            .client
            .post(url)
            .body(json! {{ "query": query }}.to_string())
            .send()
            .await?;
        log::trace!("http response status: {}", response.status());
        let body = response.text().await?;
        log::trace!("http response body: {}", body);
        match serde_json::from_str(&body)? {
            GraphQLResponse::Success { data, .. } => Ok(data),
            GraphQLResponse::Error { errors } => Err(Error::GraphQL(errors)),
        }
    }

    pub async fn get_order(&self, order_number: OrderNumber) -> Result<Order, Error> {
        #[derive(Deserialize)]
        struct Response {
            orders: Connection<Order>,
        }

        let mut response: Response = self
            .post_graphql(format!(
                r#"
            query GetOrder {{
                orders(first: 1, query: "name:'{order_number}'") {{
                    nodes {{
                        lineItems(first: 30) {{
                            nodes {{
                                sku
                                quantity
                            }}
                        }}
                    }}
                }}
            }}
            "#,
            ))
            .await?;
        if response.orders.nodes.is_empty() {
            Err(Error::Custom(format!("Order {order_number} was not found. You will find your order number in the email you receive from our shop.")))
        } else {
            Ok(response.orders.nodes.remove(0))
        }
    }
}

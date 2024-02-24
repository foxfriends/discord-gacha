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
            "X-Client-Access-Token",
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
        let response = self
            .client
            .post(url)
            .body(json! {{ "query": query }}.to_string())
            .send()
            .await?;
        match response.json().await {
            Ok(GraphQLResponse::Success { data }) => Ok(data),
            Ok(GraphQLResponse::Error { errors }) => Err(Error::GraphQL(errors)),
            Err(..) => Err(Error::Json),
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
                orders(query: "name={order_number}") {{
                    nodes {{
                        lineItems {{
                            sku
                            quantity
                        }}
                    }}
                }}
            }}
            "#,
            ))
            .await?;
        Ok(response.orders.nodes.remove(0))
    }
}

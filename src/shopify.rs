use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;

pub struct Shopify {
    shop: String,
    client: Client,
}

impl Shopify {
    pub fn new(shop: impl AsRef<str>, token: impl AsRef<str>) -> Result<Self, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Shopify-Access-Token",
            HeaderValue::from_str(token.as_ref()).unwrap(),
        );
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        Ok(Self {
            shop: shop.as_ref().to_owned(),
            client: Client::builder().default_headers(headers).build()?,
        })
    }
}

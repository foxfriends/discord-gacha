use super::{Pool, Pools};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Products {
    product: Vec<Product>,
}

impl Products {
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn into_pools(self) -> Pools {
        Pools(
            self.product
                .into_iter()
                .fold(HashMap::new(), |mut map, product| {
                    map.entry(product.pool).or_default().push(product);
                    map
                }),
        )
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Product {
    pub name: String,
    pub sku: String,
    pub pool: Pool,
    pub rarity: f64,
}

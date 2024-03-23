use super::Pool;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{self, Display};

#[derive(Clone, Deserialize, Debug)]
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

#[derive(Clone, Deserialize, Debug)]
pub struct Product {
    pub name: String,
    pub sku: String,
    pub pool: Pool,
    pub rarity: usize,
}

#[derive(Clone, Debug)]
pub struct Pools(HashMap<Pool, Vec<Product>>);

impl Pools {
    pub fn distribution(&self) -> impl Display + '_ {
        Distribution(self)
    }
}

struct Distribution<'a>(&'a Pools);

impl Display for Distribution<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (pool, products) in &self.0 .0 {
            let total: usize = products.iter().map(|product| product.rarity).sum();
            writeln!(f, "{pool} - Total weight: {total}")?;
            for product in products {
                let percentage = ((product.rarity as f64 / total as f64) * 100.0).round();
                writeln!(f, "\t({}) {}: {}%", product.rarity, product.name, percentage)?;
            }
        }
        Ok(())
    }
}

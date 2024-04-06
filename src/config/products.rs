use super::{Banner, Pool};
use rand::distributions::weighted::WeightedIndex;
use rand::distributions::Distribution as _;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Products {
    ticket: Vec<Ticket>,
    product: Vec<Product>,
}

impl Products {
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Product> + '_ {
        self.product.iter()
    }

    pub fn distribution(&self) -> impl Display + '_ {
        Distribution(self)
    }

    pub fn banner(&self, inventory: &HashMap<String, usize>) -> Banner {
        let mut omit = HashSet::new();
        let mut rng = thread_rng();

        let products = std::array::from_fn(|_| {
            let pool: Vec<_> = self
                .product
                .iter()
                .filter(|item| inventory.get(&item.sku).copied().unwrap_or(0) > 0)
                .filter(|item| !omit.contains(&item.sku))
                .collect();
            let weights = pool.iter().map(|product| product.rarity);
            let weighted = WeightedIndex::new(weights).unwrap();
            let index = weighted.sample(&mut rng);
            let product = pool[index];
            omit.insert(&product.sku);
            product.clone()
        });
        Banner::new(products)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Ticket {
    pub sku: String,
    #[serde(default)]
    pub bulks: usize,
    #[serde(default)]
    pub singles: usize,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Product {
    pub name: String,
    pub sku: String,
    pub pool: Pool,
    pub rarity: f64,
}

struct Distribution<'a>(&'a Products);

impl Display for Distribution<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let total: f64 = self.0.product.iter().map(|product| product.rarity).sum();
        writeln!(f, "Total products: {}", self.0.product.len())?;
        for product in &self.0.product {
            let percentage = ((product.rarity / total) * 10000.0).round() / 100.0;
            writeln!(
                f,
                "\t({}) {}: {}%",
                product.rarity, product.name, percentage
            )?;
        }
        Ok(())
    }
}

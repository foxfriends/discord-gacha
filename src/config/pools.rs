use super::{Pool, Product};
use rand::distributions::weighted::WeightedIndex;
use rand::distributions::Distribution as _;
use rand::prelude::*;
use std::collections::HashMap;
use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub struct Pools(pub(super) HashMap<Pool, Vec<Product>>);

impl Pools {
    pub fn distribution(&self) -> impl Display + '_ {
        Distribution(self)
    }

    pub fn pull(&self, pool: Pool, inventory: HashMap<String, usize>) -> &Product {
        let pool = self.0.get(&pool).unwrap();
        let pool: Vec<_> = pool
            .iter()
            .filter(|item| inventory.get(&item.sku).copied().unwrap_or(0) > 0)
            .collect();
        let weights = pool.iter().map(|product| product.rarity);
        let weighted = WeightedIndex::new(weights).unwrap();
        let mut rng = thread_rng();
        let index = weighted.sample(&mut rng);
        pool[index]
    }
}

struct Distribution<'a>(&'a Pools);

impl Display for Distribution<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (pool, products) in &self.0 .0 {
            let total: f64 = products.iter().map(|product| product.rarity).sum();
            writeln!(f, "{pool} - Total weight: {total}")?;
            for product in products {
                let percentage = ((product.rarity / total) * 100.0).round();
                writeln!(
                    f,
                    "\t({}) {}: {}%",
                    product.rarity, product.name, percentage
                )?;
            }
        }
        Ok(())
    }
}

use super::{Pool, Product};
use image::imageops::{overlay, FilterType};
use image::{load_from_memory_with_format, ImageError, ImageFormat};
use serde::{Deserialize, Serialize};

const SUMMON_PNG: &[u8] = include_bytes!("../../assets/summon.png");
const RED_PNG: &[u8] = include_bytes!("../../assets/red.png");
const BLUE_PNG: &[u8] = include_bytes!("../../assets/blue.png");
const GREEN_PNG: &[u8] = include_bytes!("../../assets/green.png");
const WHITE_PNG: &[u8] = include_bytes!("../../assets/grey.png");

const ROBIN_PNG: &[u8] = include_bytes!("../../assets/Robin.png");

#[derive(Serialize, Deserialize, Debug)]
pub struct Banner {
    pub pools: Vec<Pool>,
    pub pulls: Vec<Option<Product>>,
}

impl Pool {
    fn image(&self) -> &'static [u8] {
        match self {
            Pool::Red => RED_PNG,
            Pool::Blue => BLUE_PNG,
            Pool::Green => GREEN_PNG,
            Pool::White => WHITE_PNG,
        }
    }
}

const X: [i64; 5] = [500, 780, 710, 295, 205];
const Y: [i64; 5] = [210, 400, 775, 775, 400];
const SIZE: i64 = 192;

impl Banner {
    pub fn pulled(&self) -> usize {
        self.pulls.iter().filter(|s| s.is_some()).count()
    }

    pub fn to_image(&self) -> Result<Vec<u8>, ImageError> {
        let mut summon = load_from_memory_with_format(SUMMON_PNG, ImageFormat::Png).unwrap();

        let robin = load_from_memory_with_format(ROBIN_PNG, ImageFormat::Png)
            .unwrap()
            .resize(180, 180, FilterType::Triangle);

        for (i, pool) in self.pools.iter().enumerate() {
            let image = load_from_memory_with_format(pool.image(), ImageFormat::Png).unwrap();
            overlay(&mut summon, &image, X[i] - SIZE / 2, Y[i] - SIZE / 2);
        }

        let mut bytes: Vec<u8> = Vec::new();
        summon.write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageOutputFormat::Png,
        )?;
        Ok(bytes)
    }
}

impl Default for Banner {
    fn default() -> Self {
        Self {
            pools: vec![Pool::Blue, Pool::White, Pool::Blue, Pool::Green, Pool::Red],
            pulls: vec![None; 5],
        }
    }
}

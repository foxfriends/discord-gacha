use super::{Pool, Product};
use image::imageops::overlay;
use image::{load, load_from_memory_with_format, ImageError, ImageFormat};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

const SUMMON_PNG: &[u8] = include_bytes!("../../assets/summon.png");
const RED_PNG: &[u8] = include_bytes!("../../assets/red.png");
const BLUE_PNG: &[u8] = include_bytes!("../../assets/blue.png");
const GREEN_PNG: &[u8] = include_bytes!("../../assets/green.png");
const WHITE_PNG: &[u8] = include_bytes!("../../assets/grey.png");

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

impl Banner {
    pub fn pulled(&self) -> usize {
        self.pulls.iter().filter(|s| s.is_some()).count()
    }

    pub fn to_image(&self) -> Result<Vec<u8>, ImageError> {
        let mut summon = load_from_memory_with_format(SUMMON_PNG, ImageFormat::Png).unwrap();

        for (i, pool) in self.pools.iter().enumerate() {
            let image = load_from_memory_with_format(pool.image(), ImageFormat::Png).unwrap();
            overlay(
                &mut summon,
                &image,
                X[i] - image.width() as i64 / 2,
                Y[i] - image.height() as i64 / 2,
            );
        }

        let assets_dir: PathBuf = "./assets".parse().unwrap();
        for (i, pull) in self.pulls.iter().enumerate() {
            if let Some(pull) = pull {
                let Ok(file) = File::open(assets_dir.join(&pull.sku).with_extension("png")) else {
                    continue;
                };
                let reader = BufReader::new(file);
                let Ok(image) = load(reader, ImageFormat::Png) else {
                    continue;
                };
                overlay(
                    &mut summon,
                    &image,
                    X[i] - image.width() as i64 / 2,
                    Y[i] - image.height() as i64 / 2,
                );
            }
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

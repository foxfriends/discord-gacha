use super::Pool;
use image::imageops::{overlay, FilterType};
use image::{load_from_memory, ImageError};
use serde::{Deserialize, Serialize};

const SUMMON_PNG: &[u8] = include_bytes!("../../assets/summon.png");
const ROBIN_PNG: &[u8] = include_bytes!("../../assets/Robin.png");

#[derive(Serialize, Deserialize, Debug)]
pub struct Banner {
    pub pools: Vec<Pool>,
    pub pulls: Vec<Option<String>>,
}

impl Banner {
    pub fn pulled(&self) -> usize {
        self.pulls.iter().filter(|s| s.is_some()).count()
    }

    pub fn to_image(&self) -> Result<Vec<u8>, ImageError> {
        let mut summon = load_from_memory(SUMMON_PNG).unwrap();
        let robin = load_from_memory(ROBIN_PNG)
            .unwrap()
            .resize(180, 180, FilterType::Triangle);
        overlay(&mut summon, &robin, 270, 90);
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

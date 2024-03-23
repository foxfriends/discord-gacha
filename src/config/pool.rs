use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum Pool {
    Red,
    Blue,
    Green,
    White,
}

impl Display for Pool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Red => "Red".fmt(f),
            Self::Blue => "Blue".fmt(f),
            Self::Green => "Green".fmt(f),
            Self::White => "White".fmt(f),
        }
    }
}

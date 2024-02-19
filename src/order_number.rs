use std::fmt::{self, Display};
use std::str::FromStr;

pub struct OrderNumber(u32);

impl FromStr for OrderNumber {
    type Err = <u32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl Display for OrderNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

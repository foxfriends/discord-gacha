use std::fmt::{self, Display};

#[derive(Debug)]
pub struct CustomError(pub String);

impl Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for CustomError {}

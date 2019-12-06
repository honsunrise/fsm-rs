use core::fmt::{Debug, Display, Formatter, Result};

#[derive(Clone)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new<T: Display>(message: T) -> Self {
        Error {
            message: message.to_string(),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        formatter.debug_tuple("Error").field(&self.message).finish()
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        formatter.write_str(&self.message)
    }
}

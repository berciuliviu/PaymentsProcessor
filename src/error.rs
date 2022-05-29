use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProcessorError(pub String);

impl fmt::Display for ProcessorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PROCESSOR ERROR: {}", self.0)
    }
}

impl Error for ProcessorError {}

pub fn p_error(error_msg: String) -> Result<(), Box<dyn Error>> {
    Err(Box::new(ProcessorError(error_msg)))
}

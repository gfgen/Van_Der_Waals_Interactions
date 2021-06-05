use std::{error::Error, fmt};

// Error for invalid input
#[derive(Debug)]
pub enum ErrorKind {
    Bound,
    ExtT,
    ExtCond,
    UnitSize,
    Reach,
    Dt,
    Particle,
}

#[derive(Debug)]
pub struct InvalidParamError {
    errors: Vec<ErrorKind>,
}

impl InvalidParamError {
    pub fn new(errors: Vec<ErrorKind>) -> Self {
        Self { errors }
    }
}

impl fmt::Display for InvalidParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InvalidInputError: {:?}", self.errors)
    }
}

impl Error for InvalidParamError {}

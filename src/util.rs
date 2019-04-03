use std::error::Error;
use std::fmt;

pub type AnyError = Box<Error>;

pub trait FoldResultsVecExt: Iterator {
    fn fold_results_vec<A, E>(&mut self) -> Result<Vec<A>, E>
    where
        Self: Iterator<Item = Result<A, E>>,
    {
        let mut ret: Vec<A> = vec![];
        for elt in self {
            match elt {
                Ok(v) => ret.push(v),
                Err(u) => return Err(u),
            }
        }
        Ok(ret)
    }
}

impl<I: Iterator> FoldResultsVecExt for I {}

#[derive(Debug)]
pub struct SimpleError {
    message: String,
}

impl std::fmt::Display for SimpleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for SimpleError {
    fn description(&self) -> &str {
        &self.message
    }
}

pub trait SimpleErrorExt {
    fn to_simple_error(&self) -> SimpleError;

    fn to_simple_error_boxed(&self) -> Box<SimpleError> {
        Box::new(self.to_simple_error())
    }
}

impl SimpleErrorExt for &str {
    fn to_simple_error(&self) -> SimpleError {
        SimpleError {
            message: self.to_string(),
        }
    }
}

impl SimpleErrorExt for String {
    fn to_simple_error(&self) -> SimpleError {
        SimpleError {
            message: self.to_string(),
        }
    }
}

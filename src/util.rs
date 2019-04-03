use std::error::Error;

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

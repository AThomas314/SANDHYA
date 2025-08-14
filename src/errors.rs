use ndarray_rand::rand_distr::{BernoulliError, NormalError, PertError, TriangularError};
use polars::error::PolarsError;
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum DistributionError {
    Normal(NormalError),
    Bernoulli(BernoulliError),
    Pert(PertError),
    Triangular(TriangularError),
}

impl From<PertError> for DistributionError {
    fn from(e: PertError) -> DistributionError {
        DistributionError::Pert(e)
    }
}
impl From<NormalError> for DistributionError {
    fn from(e: NormalError) -> DistributionError {
        DistributionError::Normal(e)
    }
}
impl From<BernoulliError> for DistributionError {
    fn from(e: BernoulliError) -> DistributionError {
        DistributionError::Bernoulli(e)
    }
}
impl From<TriangularError> for DistributionError {
    fn from(e: TriangularError) -> DistributionError {
        DistributionError::Triangular(e)
    }
}
impl From<DistributionError> for PolarsError {
    fn from(err: DistributionError) -> Self {
        // We wrap your error's text inside a PolarsError::ComputeError
        PolarsError::ComputeError(err.to_string().into())
    }
}

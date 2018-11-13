mod bimap;
pub use bimap::WordMap;

mod collector;
pub use collector::{Collector, TupleCollector};

mod mi;
pub use mi::{
    LaplaceSmoothing, MutualInformation, PositiveMutualInformation, RawProb, Smoothing,
    SpecificCorrelation,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

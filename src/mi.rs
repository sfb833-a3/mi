use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

/// Trait for mutual information measures.
pub trait MutualInformation<V>
where
    V: Eq + Hash,
{
    fn mutual_information(
        &self,
        tuple: &[V],
        event_freqs: &[HashMap<V, usize>],
        event_sums: &[usize],
        joint_freq: usize,
        joint_sum: usize,
    ) -> f64;
}

/// Specific correlation (Van de Cruys, 2011)
///
/// The specific correlation measure is a generalization of PMI for multiple
/// random variables.
pub struct SpecificCorrelation {
    normalize: bool,
}

impl SpecificCorrelation {
    /// Construct a new specific correlation function.
    ///
    /// If normalization is enable, the result will lie between -1 and 1.
    pub fn new(normalize: bool) -> Self {
        SpecificCorrelation {
            normalize: normalize,
        }
    }
}

impl<V> MutualInformation<V> for SpecificCorrelation
where
    V: Eq + Hash,
{
    fn mutual_information(
        &self,
        tuple: &[V],
        event_freqs: &[HashMap<V, usize>],
        event_sums: &[usize],
        joint_freq: usize,
        joint_sum: usize,
    ) -> f64 {
        let tuple_len = tuple.as_ref().len();
        let pmi = sc(tuple, event_freqs, event_sums, joint_freq, joint_sum);

        if self.normalize {
            let tuple_p = joint_freq as f64 / joint_sum as f64;

            if pmi.is_sign_positive() {
                pmi / (-((tuple_len - 1) as f64) * tuple_p.ln())
            } else {
                pmi / -tuple_p.ln()
            }
        } else {
            pmi
        }
    }
}

/// Positive mutual information.
///
/// This function is a simple wrapper around anouther mutual information
/// function that will 'round' negative mutual information to *0*.
pub struct PositiveMutualInformation<M, V>
where
    M: MutualInformation<V>,
    V: Eq + Hash,
{
    mi: M,
    tuple_value_type: PhantomData<V>,
}

impl<M, V> PositiveMutualInformation<M, V>
where
    M: MutualInformation<V>,
    V: Eq + Hash,
{
    pub fn new(mi: M) -> Self {
        PositiveMutualInformation {
            mi: mi,
            tuple_value_type: PhantomData,
        }
    }
}

impl<M, V> MutualInformation<V> for PositiveMutualInformation<M, V>
where
    M: MutualInformation<V>,
    V: Eq + Hash,
{
    fn mutual_information(
        &self,
        tuple: &[V],
        event_freqs: &[HashMap<V, usize>],
        event_sums: &[usize],
        joint_freq: usize,
        joint_sum: usize,
    ) -> f64 {
        let score =
            self.mi
                .mutual_information(tuple, event_freqs, event_sums, joint_freq, joint_sum);
        if score < 0f64 {
            0f64
        } else {
            score
        }
    }
}

fn sc<V>(
    tuple: &[V],
    event_freqs: &[HashMap<V, usize>],
    event_sums: &[usize],
    joint_freq: usize,
    joint_sum: usize,
) -> f64
where
    V: Eq + Hash,
{
    let tuple_p = joint_freq as f64 / joint_sum as f64;

    let indep_p = tuple
        .as_ref()
        .iter()
        .enumerate()
        .map(|(idx, v)| {
            event_freqs[idx][v] as f64 / event_sums[idx] as f64
        })
        .fold(1.0, |acc, v| acc * v);

    (tuple_p / indep_p).ln()
}

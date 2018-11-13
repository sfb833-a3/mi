use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

/// Trait for smoothing methods.
pub trait Smoothing<V> {
    fn joint_prob(&self, joint_freq: usize, joint_sum: usize, joint_freqs_len: usize) -> f64;
    fn prob(&self, tuple: &[V], event_freqs: &[HashMap<V, usize>], event_sums: &[usize]) -> f64;
}

impl<T, V> Smoothing<V> for Box<T> where T: ?Sized + Smoothing<V> {
    fn joint_prob(&self, joint_freq: usize, joint_sum: usize, joint_freqs_len: usize) -> f64 {
        self.as_ref().joint_prob(joint_freq, joint_sum, joint_freqs_len)
    }

    fn prob(&self, tuple: &[V], event_freqs: &[HashMap<V, usize>], event_sums: &[usize]) -> f64 {
        self.as_ref().prob(tuple, event_freqs, event_sums)
    }
}

/// Laplace smoothing
pub struct LaplaceSmoothing {
    alpha: f64,
}

impl LaplaceSmoothing where {
    pub fn new(alpha: f64) -> Self {
        LaplaceSmoothing { alpha }
    }
}

impl<V> Smoothing<V> for LaplaceSmoothing
where
    V: Eq + Hash,
{
    fn prob(&self, tuple: &[V], event_freqs: &[HashMap<V, usize>], event_sums: &[usize]) -> f64 {
        tuple
            .as_ref()
            .iter()
            .enumerate()
            .map(|(idx, v)| {
                (event_freqs[idx][v] as f64 + self.alpha)
                    / (event_sums[idx] + event_freqs[idx].len()) as f64
            }).fold(1.0, |acc, v| acc * v)
    }

    fn joint_prob(&self, joint_freq: usize, joint_sum: usize, joint_freqs_len: usize) -> f64 {
        (joint_freq as f64 + self.alpha) / (joint_sum as f64 + joint_freqs_len as f64 * self.alpha)
    }
}

/// Compute probabilities without smoothing
pub struct RawProb;

impl RawProb where {
    pub fn new() -> Self {
        RawProb
    }
}

impl<V> Smoothing<V> for RawProb
where
    V: Eq + Hash,
{
    fn prob(&self, tuple: &[V], event_freqs: &[HashMap<V, usize>], event_sums: &[usize]) -> f64 {
        tuple
            .as_ref()
            .iter()
            .enumerate()
            .map(|(idx, v)| event_freqs[idx][v] as f64 / event_sums[idx] as f64)
            .fold(1.0, |acc, v| acc * v)
    }
    fn joint_prob(&self, joint_freq: usize, joint_sum: usize, _joint_freqs_len: usize) -> f64 {
        joint_freq as f64 / joint_sum as f64
    }
}

/// Trait for mutual information measures.
pub trait MutualInformation<V> {
    fn mutual_information(
        &self,
        tuple: &[V],
        event_freqs: &[HashMap<V, usize>],
        event_sums: &[usize],
        num_joint_events: usize,
        joint_freq: usize,
        joint_sum: usize,
    ) -> f64;
}

/// Specific correlation (Van de Cruys, 2011)
///
/// The specific correlation measure is a generalization of PMI for multiple
/// random variables.
pub struct SpecificCorrelation<S> {
    normalize: bool,
    smoothing: S,
}

impl<S> SpecificCorrelation<S> {
    /// Construct a new specific correlation function.
    ///
    /// If normalization is enable, the result will lie between -1 and 1.
    pub fn new(normalize: bool, smoothing: S) -> Self {
        SpecificCorrelation {
            normalize,
            smoothing,
        }
    }
}

impl<S, V> MutualInformation<V> for SpecificCorrelation<S> where S: Smoothing<V> {
    fn mutual_information(
        &self,
        tuple: &[V],
        event_freqs: &[HashMap<V, usize>],
        event_sums: &[usize],
        joint_freqs_len: usize,
        joint_freq: usize,
        joint_sum: usize,
    ) -> f64 {
        let tuple_len = tuple.as_ref().len();
        let pmi = sc(
            tuple,
            event_freqs,
            event_sums,
            joint_freqs_len,
            joint_freq,
            joint_sum,
            &self.smoothing,
        );

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
/// This function is a simple wrapper around another mutual information
/// function that will 'round' negative mutual information to *0*.
pub struct PositiveMutualInformation<M, V>
where
    M: MutualInformation<V>,
{
    mi: M,
    tuple_value_type: PhantomData<V>,
}

impl<M, V> PositiveMutualInformation<M, V>
where
    M: MutualInformation<V>,
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
{
    fn mutual_information(
        &self,
        tuple: &[V],
        event_freqs: &[HashMap<V, usize>],
        event_sums: &[usize],
        num_joint_events: usize,
        joint_freq: usize,
        joint_sum: usize,
    ) -> f64 {
        let score = self.mi.mutual_information(
            tuple,
            event_freqs,
            event_sums,
            num_joint_events,
            joint_freq,
            joint_sum,
        );
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
    joint_freqs_len: usize,
    joint_freq: usize,
    joint_sum: usize,
    smoothing: &Smoothing<V>,
) -> f64 {
    let tuple_p = smoothing.joint_prob(joint_freq, joint_sum, joint_freqs_len);
    let indep_p = smoothing.prob(tuple, event_freqs, event_sums);

    (tuple_p / indep_p).ln()
}

#[cfg(test)]
mod tests {
    use super::{LaplaceSmoothing, Smoothing, RawProb};
    use tests::EVENT_FREQS;

    const TUPLE: &[usize] = &[1, 2];
    const EVENT_SUMS: &[usize] = &[12, 11];
    const JOINT_FREQS_LEN: usize = 4;
    const JOINT_FREQ: usize = 2;
    const JOINT_SUM: usize = 11;

    #[test]
    pub fn test_laplace() {
        let smoothing = LaplaceSmoothing::new(1_f64);

        let tuple_p = smoothing.joint_prob(JOINT_FREQ, JOINT_SUM, JOINT_FREQS_LEN);
        let indep_p = smoothing.prob(TUPLE, &EVENT_FREQS, EVENT_SUMS);

        let res = (tuple_p / indep_p).ln();
        let cmp = 0.667829373;
        assert!((res - cmp).abs() < 1e-5);
    }

    #[test]
    pub fn test_rawprob() {
        let smoothing = RawProb::new();

        let tuple_p = smoothing.joint_prob(JOINT_FREQ, JOINT_SUM, JOINT_FREQS_LEN);
        let indep_p = smoothing.prob(TUPLE, &EVENT_FREQS, EVENT_SUMS);

        let res = (tuple_p / indep_p).ln();
        let cmp = 0.693147181;
        assert!((res - cmp).abs() < 1e-5);
    }
}

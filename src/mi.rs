use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

#[derive(PartialEq)]
pub enum Smoothing {
    /// Do not apply smoothing
    None,
    /// Apply smoothing as described in Jurafsky and Martin, ch.6, pp.18f
    Alpha,
    /// Apply Laplacian smoothing
    Laplace
}

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
        joint_freq_freq: usize,
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
    smoothing: Smoothing,
}

impl SpecificCorrelation {
    /// Construct a new specific correlation function.
    ///
    /// If normalization is enable, the result will lie between -1 and 1.
    pub fn new(normalize: bool, smoothing: Smoothing) -> Self {
        SpecificCorrelation {
            normalize,
            smoothing
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
        joint_freq_freq: usize,
        joint_freq: usize,
        joint_sum: usize,
    ) -> f64 {
        let tuple_len = tuple.as_ref().len();
        let pmi = sc(tuple, event_freqs, event_sums, joint_freq_freq, joint_freq, joint_sum, &self.smoothing);

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
        joint_freq_freq: usize,
        joint_freq: usize,
        joint_sum: usize,
    ) -> f64 {
        let score =
            self.mi
                .mutual_information(tuple, event_freqs, event_sums, joint_freq_freq, joint_freq, joint_sum);
        if score < 0f64 {
            0f64
        } else {
            score
        }
    }
}

fn sc<V>(
    tuple: &[V],
    event_freqs: &[HashMap<V, usize>],  // How often does each word occur?
    event_sums: &[usize],   // How many words occur at this position in total?
    joint_freq_freq: usize,  // How many different tuples do we have?
    joint_freq: usize,  // How often does a tuple occur?
    joint_sum: usize,   // How many tuples of this size do we have?
    smoothing: &Smoothing
) -> f64
    where
        V: Eq + Hash,
{
    let tuple_p = match smoothing {
        &Smoothing::Laplace => (joint_freq + 1) as f64 / (joint_sum + joint_freq_freq) as f64,
        _ => joint_freq as f64 / joint_sum as f64
    };

    let indep_p = match smoothing {
        &Smoothing::Laplace => {
            let mut num_events = 0;
            for i in 0..event_freqs.len() {
                num_events = num_events + event_freqs[i].len();
            };
            tuple
                .as_ref()
                .iter()
                .enumerate()
                .map(|(idx, v)| {
                    (event_freqs[idx][v] + 1) as f64 / (event_sums[idx] + num_events) as f64
                })
                .fold(1.0, |acc, v| acc * v)
        },
        &Smoothing::Alpha => {
            let mut num_events = 0.0;
            // Raise each different event to powf(0.75)
            for i in 0..event_freqs.len() {
                num_events = num_events + (event_freqs[i].len() as f64).powf(0.75);
            }
            tuple
                .as_ref()
                .iter()
                .enumerate()
                .map(|(idx, v)| {   // The focus word is also raised to powf(0.75) here though only context words should be powf
                    (event_freqs[idx][v] as f64).powf(0.75) / num_events
                })
                .fold(1.0, |acc, v| acc * v)
        },
        &Smoothing::None => tuple
            .as_ref()
            .iter()
            .enumerate()
            .map(|(idx, v)| {
                event_freqs[idx][v] as f64 / event_sums[idx] as f64
            })
            .fold(1.0, |acc, v| acc * v)
    };

    (tuple_p / indep_p).ln()
}
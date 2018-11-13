use std::collections::hash_map;
use std::collections::HashMap;
use std::hash::Hash;

use super::MutualInformation;

/// Trait for frequency collectors for mutual information computations.
pub trait Collector<V> {
    /// Count an observation.
    fn count(&mut self, slice: &[V]);

    /// Iterate over all observations, their frequencies, and mutual
    /// information scores as defined by the provided mutual information
    /// function.
    fn iter<'a>(
        &'a self,
        mi: &'a MutualInformation<V>,
    ) -> Box<Iterator<Item = (&'a [V], usize, f64)> + 'a>
    where
        V: Eq + Hash;
}

pub struct TupleCollector<T, V> {
    event_freqs: Vec<HashMap<V, usize>>,
    event_sums: Vec<usize>,
    joint_freqs: HashMap<T, usize>,
    joint_sum: usize,
}

/// A memory-efficient `Collector` of observations.
///
/// `TupleCollector` stores observations as tuples (fixed-length arrays).
/// This representation does not have the memory overhead of e.g. `Vec`.
///
/// This collector collects the distribution per position. So, for a
/// 3-triple, separate counts are kept for events in the first, second,
/// and third position.
impl<T, V> Collector<V> for TupleCollector<T, V>
where
    T: AsMut<[V]> + AsRef<[V]> + Clone + Default + Eq + Hash,
    V: Clone + Eq + Hash,
{
    fn count(&mut self, slice: &[V]) {
        let mut tuple = T::default();

        {
            let tuple_ref = tuple.as_mut();

            assert!(
                tuple_ref.len() == slice.as_ref().len(),
                format!(
                    "Attempting to add slice of size {} to collector of size {}",
                    slice.as_ref().len(),
                    tuple_ref.len()
                )
            );

            tuple_ref.clone_from_slice(slice.as_ref());
        }

        self.count_tuple(tuple);
    }

    fn iter<'a>(
        &'a self,
        mi: &'a MutualInformation<V>,
    ) -> Box<Iterator<Item = (&'a [V], usize, f64)> + 'a> {
        Box::new(Iter {
            collector: self,
            inner: self.joint_freqs.iter(),
            mi: mi,
        })
    }
}

impl<T, V> TupleCollector<T, V>
where
    T: AsRef<[V]> + Clone + Default + Eq + Hash,
    V: Clone + Eq + Hash,
{
    /// Construct a new `ColumnTupleCollector`.
    pub fn new() -> Self {
        let tuple_len = T::default().as_ref().len();

        TupleCollector {
            event_freqs: vec![HashMap::new(); tuple_len],
            event_sums: vec![0; tuple_len],
            joint_freqs: HashMap::new(),
            joint_sum: 0,
        }
    }

    fn count_tuple(&mut self, tuple: T) {
        for (idx, v) in tuple.as_ref().iter().enumerate() {
            *self.event_freqs[idx].entry(v.clone()).or_insert(0) += 1;
            self.event_sums[idx] += 1;
        }

        *self.joint_freqs.entry(tuple).or_insert(0) += 1;
        self.joint_sum += 1;
    }
}

pub struct Iter<'a, T, V>
where
    T: 'a + AsRef<[V]>,
    V: 'a + Eq + Hash,
{
    collector: &'a TupleCollector<T, V>,
    inner: hash_map::Iter<'a, T, usize>,
    mi: &'a MutualInformation<V>,
}

impl<'a, T, V> Iterator for Iter<'a, T, V>
where
    T: AsRef<[V]> + Clone + Eq + Hash,
    V: Eq + Hash,
{
    type Item = (&'a [V], usize, f64);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(tuple, tuple_count)| {
            let mi = self.mi.mutual_information(
                tuple.as_ref(),
                &self.collector.event_freqs,
                &self.collector.event_sums,
                self.collector.joint_freqs.len(),
                *tuple_count,
                self.collector.joint_sum,
            );

            (tuple.as_ref(), *tuple_count, mi)
        })
    }
}

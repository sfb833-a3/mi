use std::collections::HashMap;
use std::collections::hash_map;
use std::hash::Hash;

use super::MutualInformation;

/// Trait for frequency collectors for mutual information computations.
pub trait Collector<V> {
    /// Count an observation.
    fn count(&mut self, slice: &[V]);

    /// Iterate over all observations, their frequencies, and mutual
    /// information scores as defined by the provided mutual information
    /// function.
    fn iter<'a>(&'a self,
                mi: &'a MutualInformation<V>)
                -> Box<Iterator<Item = (&'a [V], usize, f64)> + 'a>
        where V: Eq + Hash;
}

/// A memory-efficient `Collector` of observations.
///
/// `TupleCollector` stores observations as tuples (fixed-length arrays).
/// This representation does not have the memory overhead of e.g. `Vec`.
pub struct TupleCollector<T, V> {
    counts: HashMap<V, usize>,
    pair_counts: HashMap<T, usize>,
    freq: usize,
}

impl<T, V> Collector<V> for TupleCollector<T, V>
    where T: AsMut<[V]> + AsRef<[V]> + Clone + Default + Eq + Hash,
          V: Clone + Eq + Hash
{
    fn count(&mut self, slice: &[V]) {
        let mut tuple = T::default();

        {
            let mut tuple_ref = tuple.as_mut();

            assert!(tuple_ref.len() == slice.as_ref().len(),
                    format!("Attempting to add slice of size {} to collector of size {}",
                            slice.as_ref().len(),
                            tuple_ref.len()));

            tuple_ref.clone_from_slice(slice.as_ref());
        }

        self.count_tuple(tuple);
    }

    fn iter<'a>(&'a self,
                mi: &'a MutualInformation<V>)
                -> Box<Iterator<Item = (&'a [V], usize, f64)> + 'a> {
        Box::new(Iter {
            collector: self,
            inner: self.pair_counts.iter(),
            mi: mi,
        })
    }
}

impl<T, V> TupleCollector<T, V>
    where T: AsRef<[V]> + Clone + Eq + Hash,
          V: Clone + Eq + Hash
{
    /// Construct a new `TupleCollector`.
    pub fn new() -> Self {
        TupleCollector {
            freq: 0,
            counts: HashMap::new(),
            pair_counts: HashMap::new(),
        }
    }

    fn count_tuple(&mut self, tuple: T) {

        for v in tuple.as_ref() {
            self.freq += 1;
            *self.counts.entry(v.clone()).or_insert(0) += 1;
        }


        *self.pair_counts.entry(tuple).or_insert(0) += 1;
    }
}

pub struct Iter<'a, T, V>
    where T: 'a + AsRef<[V]>,
          V: 'a + Eq + Hash
{
    collector: &'a TupleCollector<T, V>,
    inner: hash_map::Iter<'a, T, usize>,
    mi: &'a MutualInformation<V>,
}

impl<'a, T, V> Iterator for Iter<'a, T, V>
    where T: AsRef<[V]> + Clone + Eq + Hash,
          V: Eq + Hash
{
    type Item = (&'a [V], usize, f64);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(tuple, tuple_count)| {

            let mi = self.mi.mutual_information(tuple.as_ref(),
                                                *tuple_count,
                                                &self.collector.counts,
                                                self.collector.freq);

            (tuple.as_ref(), *tuple_count, mi)
        })
    }
}

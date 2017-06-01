use std::collections::HashMap;
use std::collections::hash_map;
use std::fmt::Display;
use std::hash::Hash;
use std::process;

mod bimap;
pub use bimap::WordMap;

pub trait OrExit {
    type RetVal;

    fn or_exit(self, msg: &str) -> Self::RetVal;
}

impl<T, E> OrExit for Result<T, E>
    where E: Display
{
    type RetVal = T;

    fn or_exit(self, msg: &str) -> Self::RetVal {
        self.unwrap_or_else(|e: E| -> T {
            println!("{}: {}", msg, e);
            process::exit(1)
        })
    }
}

#[derive(Clone, Copy)]
pub enum MutualInformation {
    NSC,
    SC,
    PSC,
}

pub trait Collector<V> {
    fn count(&mut self, slice: &[V]);

    fn iter<'a>(&'a self,
                mi: MutualInformation)
                -> Box<Iterator<Item = (&'a [V], usize, f64)> + 'a>;
}

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

            assert!(tuple_ref.len() == slice.len(),
                    format!("Attempting to add slice of size {} to collector of size {}",
                            slice.len(),
                            tuple_ref.len()));

            tuple_ref.clone_from_slice(slice);
        }

        self.count_tuple(tuple);
    }

    fn iter<'a>(&'a self,
                mi: MutualInformation)
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
    where T: 'a,
          V: 'a
{
    collector: &'a TupleCollector<T, V>,
    inner: hash_map::Iter<'a, T, usize>,
    mi: MutualInformation,
}

impl<'a, T, V> Iterator for Iter<'a, T, V>
    where T: AsRef<[V]> + Clone + Eq + Hash,
          V: Eq + Hash
{
    type Item = (&'a [V], usize, f64);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(tuple, tuple_count)| {

            let mi = mutual_information(self.mi,
                                        &tuple,
                                        *tuple_count,
                                        &self.collector.counts,
                                        self.collector.freq);

            (tuple.as_ref(), *tuple_count, mi)
        })
    }
}

fn mutual_information<T, V>(measure: MutualInformation,
                            tuple: T,
                            tuple_freq: usize,
                            freqs: &HashMap<V, usize>,
                            freq: usize)
                            -> f64
    where T: AsRef<[V]>,
          V: Eq + Hash
{
    let f = match measure {
        MutualInformation::NSC => nsc,
        MutualInformation::SC => sc,
        MutualInformation::PSC => psc,
    };

    f(tuple, tuple_freq, freqs, freq)
}

fn nsc<T, V>(tuple: T, tuple_freq: usize, freqs: &HashMap<V, usize>, freq: usize) -> f64
    where T: AsRef<[V]>,
          V: Eq + Hash
{
    let tuple_len = tuple.as_ref().len();

    let pmi = sc(tuple, tuple_freq, freqs, freq);

    let pair_p = tuple_freq as f64 / freq as f64;

    if pmi.is_sign_positive() {
        pmi / (-((tuple_len - 1) as f64) * pair_p.ln())
    } else {
        pmi / -pair_p.ln()
    }
}

fn psc<T, V>(tuple: T, tuple_freq: usize, freqs: &HashMap<V, usize>, freq: usize) -> f64
    where T: AsRef<[V]>,
          V: Eq + Hash
{
    let pmi = sc(tuple, tuple_freq, freqs, freq);

    if pmi < 0f64 { 0f64 } else { pmi }
}

fn sc<T, V>(tuple: T, tuple_freq: usize, freqs: &HashMap<V, usize>, freq: usize) -> f64
    where T: AsRef<[V]>,
          V: Eq + Hash
{
    let pair_p = tuple_freq as f64 / freq as f64;

    let indep_p =
        tuple.as_ref().iter().map(|v| freqs[v] as f64 / freq as f64).fold(1.0, |acc, v| acc * v);

    (pair_p / indep_p).ln()
}

#[macro_export]
macro_rules! stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

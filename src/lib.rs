use std::collections::HashMap;
use std::collections::hash_map;
use std::fmt::Display;
use std::hash::Hash;
use std::mem;
use std::process;

mod bimap;
pub use bimap::WordMap;

#[derive(Clone, Copy)]
pub enum MutualInformation {
    NSC,
    SC,
    PSC,
}

pub struct Collector<T, V> {
    counts: HashMap<V, usize>,
    pair_counts: HashMap<T, usize>,
    freq: usize,
}

impl<T, V> Collector<T, V>
    where T: AsRef<[V]> + Clone + Eq + Hash,
          V: Clone + Eq + Hash
{
    pub fn new() -> Self {
        Collector {
            freq: 0,
            counts: HashMap::new(),
            pair_counts: HashMap::new(),
        }
    }

    pub fn count(&mut self, tuple: T) {

        for v in tuple.as_ref() {
            self.freq += 1;
            *self.counts.entry(v.clone()).or_insert(0) += 1;
        }


        *self.pair_counts.entry(tuple).or_insert(0) += 1;
    }

    pub fn filter_freq(&mut self, cutoff: usize) {
        let mut counts = HashMap::new();
        mem::swap(&mut self.pair_counts, &mut counts);

        for (pair, count) in counts {
            if count >= cutoff {
                self.pair_counts.insert(pair, count);
            }
        }
    }

    pub fn iter(&self, mi: MutualInformation) -> Iter<T, V> {
        Iter {
            collector: self,
            inner: self.pair_counts.iter(),
            mi: mi,
        }
    }
}

pub struct Iter<'a, T, V>
    where T: 'a,
          V: 'a
{
    collector: &'a Collector<T, V>,
    inner: hash_map::Iter<'a, T, usize>,
    mi: MutualInformation,
}

impl<'a, T, V> Iterator for Iter<'a, T, V>
    where T: AsRef<[V]> + Clone + Eq + Hash,
          V: Eq + Hash
{
    type Item = (&'a T, f64);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(tuple, pair_count)| {

            let mi = mutual_information(self.mi,
                                        &tuple,
                                        *pair_count,
                                        &self.collector.counts,
                                        self.collector.freq);

            (tuple, mi)
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
    let pmi = sc(tuple, tuple_freq, freqs, freq);

    let pair_p = tuple_freq as f64 / freq as f64;

    if pmi.is_sign_positive() {
        pmi / (-2.0 * pair_p.ln())
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

pub fn or_exit<T, E: Display>(r: Result<T, E>) -> T {
    r.unwrap_or_else(|e: E| -> T {
        println!("Error: {}", e);
        process::exit(1)
    })
}

#[macro_export]
macro_rules! stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

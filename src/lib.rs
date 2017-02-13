use std::collections::HashMap;
use std::collections::hash_map;
use std::fmt::Display;
use std::hash::Hash;
use std::mem;
use std::process;

pub enum MutualInformation {
    NPMI,
    PMI,
    PPMI,
}

#[derive(Clone, Copy)]
pub enum Smoothing {
    None,
    Add(usize),
}

pub struct Collector<T> {
    counts: HashMap<T, usize>,
    pair_counts: HashMap<(T, T), usize>,
    freq: usize,
}

impl<T> Collector<T>
    where T: Clone + Eq + Hash
{
    pub fn new() -> Self {
        Collector {
            freq: 0,
            counts: HashMap::new(),
            pair_counts: HashMap::new(),
        }
    }

    pub fn count(&mut self, v1: T, v2: T) {
        self.freq += 2;
        *self.counts.entry(v1.clone()).or_insert(0) += 1;
        *self.counts.entry(v2.clone()).or_insert(0) += 1;
        *self.pair_counts.entry((v1, v2)).or_insert(0) += 1;
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

    pub fn iter(&self, mi: MutualInformation, smoothing: Smoothing) -> Iter<T> {
        Iter {
            collector: self,
            inner: self.pair_counts.iter(),
            mi: mi,
            smoothing: smoothing,
        }

    }
}

pub struct Iter<'a, T>
    where T: 'a
{
    collector: &'a Collector<T>,
    inner: hash_map::Iter<'a, (T, T), usize>,
    mi: MutualInformation,
    smoothing: Smoothing,
}

impl<'a, T> Iterator for Iter<'a, T>
    where T: Eq + Hash
{
    type Item = ((&'a T, &'a T), f64);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(&(ref v1, ref v2), pair_count)| {

            let mi = match self.mi {
                MutualInformation::NPMI => {
                    npmi(*pair_count,
                         self.collector.counts[v1],
                         self.collector.counts[v2],
                         self.collector.freq,
                         self.smoothing)
                }
                MutualInformation::PMI => {
                    pmi(*pair_count,
                        self.collector.counts[v1],
                        self.collector.counts[v2],
                        self.collector.freq,
                        self.smoothing)
                }
                MutualInformation::PPMI => {
                    ppmi(*pair_count,
                         self.collector.counts[v1],
                         self.collector.counts[v2],
                         self.collector.freq,
                         self.smoothing)
                }
            };
            ((v1, v2), mi)
        })
    }
}

fn npmi(pair_freq: usize,
        v1_freq: usize,
        v2_freq: usize,
        freq: usize,
        smoothing: Smoothing)
        -> f64 {
    let pmi = pmi(pair_freq, v1_freq, v2_freq, freq, smoothing);

    let add = match smoothing {
        Smoothing::None => 0,
        Smoothing::Add(n) => n,
    };

    let pair_p = (pair_freq + add) as f64 / freq as f64;

    pmi / -pair_p.ln()
}

fn ppmi(pair_freq: usize,
        v1_freq: usize,
        v2_freq: usize,
        freq: usize,
        smoothing: Smoothing)
        -> f64 {
    let pmi = pmi(pair_freq, v1_freq, v2_freq, freq, smoothing);

    if pmi < 0f64 { 0f64 } else { pmi }
}

fn pmi(pair_freq: usize, v1_freq: usize, v2_freq: usize, freq: usize, smoothing: Smoothing) -> f64 {
    let add = match smoothing {
        Smoothing::None => 0,
        Smoothing::Add(n) => n,
    };

    let pair_p = (pair_freq + add) as f64 / freq as f64;
    let v1_p = (v1_freq + add) as f64 / freq as f64;
    let v2_p = (v2_freq + add) as f64 / freq as f64;

    (pair_p / (v1_p * v2_p)).ln()
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

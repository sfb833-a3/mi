extern crate getopts;

extern crate itertools;

#[macro_use]
extern crate mi;
extern crate stdinout;

use std::collections::HashSet;
use std::env::args;
use std::hash::Hash;
use std::io::{BufRead, BufWriter, Write};
use std::process;

use getopts::Options;
use itertools::Itertools;
use mi::{OrExit, SpecificCorrelation};
use stdinout::*;

use mi::*;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] VARS INPUT OUTPUT", program);
    print!("{}", opts.usage(&brief));
}

fn measure_from_str<V>(measure_str: &str) -> Box<MutualInformation<V>>
where
    V: 'static + Eq + Hash,
{
    match measure_str {
        "sc" => Box::new(SpecificCorrelation::new(false)),
        "nsc" => Box::new(SpecificCorrelation::new(true)),
        "psc" => Box::new(PositiveMutualInformation::new(
            SpecificCorrelation::new(false),
        )),
        "pnsc" => Box::new(PositiveMutualInformation::new(
            SpecificCorrelation::new(true),
        )),
        _ => {
            stderr!("Unknown mutual information measure: {}", measure_str);
            process::exit(1);
        }
    }
}

fn parse_indices(indices_str: &str) -> HashSet<usize> {
    let mut indices = HashSet::new();

    for part in indices_str.split(',') {
        let idx: usize = part.parse().or_exit("Cannot parse index");

        if idx == 0 {
            stderr!("Bad index: variable argument uses 1-based indexing");
            process::exit(1);
        }

        indices.insert(idx - 1);
    }

    indices
}

fn main() {
    let args: Vec<String> = args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("f", "freq", "minimum frequency cut-off", "N");
    opts.optopt(
        "m",
        "measure",
        "mutual information measure: sc, nsc, psc, or pnsc (default: sc)",
        "MEASURE",
    );
    opts.optflag("h", "help", "print this help menu");

    let matches = opts.parse(&args[1..]).or_exit("Cannot parse arguments");

    if matches.opt_present("h") {
        print_usage(&program, opts);
        process::exit(1)
    }

    let cutoff = matches
        .opt_str("f")
        .map(|v| v.parse().or_exit("Cannot parse frequency"))
        .unwrap_or(1);

    let measure = measure_from_str(&matches.opt_str("m").unwrap_or("sc".to_owned()));

    if matches.free.len() == 0 || matches.free.len() > 3 {
        print_usage(&program, opts);
        process::exit(1)
    }

    let indices = parse_indices(&matches.free[0]);

    let input = Input::from(matches.free.get(1));
    let reader = input.buf_read().or_exit("Cannot open reader");

    let output = Output::from(matches.free.get(2));
    let mut writer = BufWriter::new(output.write().or_exit("Cannot open writer"));

    let mut word_map = WordMap::new();
    let mut collector: Box<Collector<usize>> = match indices.len() {
        2 => Box::new(TupleCollector::new() as TupleCollector<[usize; 2], usize>),
        3 => Box::new(TupleCollector::new() as TupleCollector<[usize; 3], usize>),
        _ => {
            stderr!("Cannot handle {} variable(s)", indices.len());
            process::exit(1);
        }
    };

    let mut tuple = Vec::with_capacity(indices.len());

    for line in reader.lines() {
        let line = line.or_exit("Cannot extract line from input");

        tuple.clear();
        for (idx, column) in line.trim().split_whitespace().enumerate() {
            if indices.contains(&idx) {
                tuple.push(word_map.number(column))
            }
        }

        collector.count(&tuple);
    }

    for (tuple, freq, pmi) in collector.iter(measure.as_ref()) {
        if freq >= cutoff {
            writeln!(
                writer,
                "{} {}",
                tuple.iter().map(|&w| word_map.word(w).unwrap()).join(" "),
                pmi
            ).or_exit("Cannot write MI to output");
        }
    }
}

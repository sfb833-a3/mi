extern crate getopts;

#[macro_use]
extern crate itertools;

#[macro_use]
extern crate mi;
extern crate stdinout;

use std::env::args;
use std::io::{BufRead, BufWriter, Write};
use std::process;

use getopts::Options;
use itertools::Itertools;
use stdinout::*;

use mi::*;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] VARS INPUT OUTPUT", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("f", "freq", "minimum frequency cut-off", "N");
    opts.optflag("h", "help", "print this help menu");

    let matches = or_exit(opts.parse(&args[1..]));

    if matches.opt_present("h") {
        print_usage(&program, opts);
        process::exit(1)
    }

    let cutoff = matches.opt_str("f").map(|v| or_exit(v.parse())).unwrap_or(1);

    if matches.free.len() == 0 || matches.free.len() > 3 {
        print_usage(&program, opts);
        process::exit(1)
    }

    let n_vars = or_exit(matches.free[0].parse());

    let input = Input::from(matches.free.get(1));
    let reader = or_exit(input.buf_read());

    let output = Output::from(matches.free.get(2));
    let mut writer = BufWriter::new(or_exit(output.write()));

    let mut word_map = WordMap::new();
    let mut collector: Box<Collector<usize>> = match n_vars {
        2 => Box::new(TupleCollector::new() as TupleCollector<[usize; 2], usize>),
        3 => Box::new(TupleCollector::new() as TupleCollector<[usize; 3], usize>),
        _ => {
            stderr!("Cannot handle {} variables", n_vars);
            process::exit(1);
        }
    };

    for line in reader.lines() {
        let line = or_exit(line);

        let tuple: Vec<_> = line.trim().split_whitespace().map(|w| word_map.number(w)).collect();

        collector.count(&tuple);
    }

    for (tuple, freq, pmi) in collector.iter(MutualInformation::NSC) {
        if freq >= cutoff {
            or_exit(writeln!(writer,
                             "{} {}",
                             tuple.iter().map(|&w| word_map.word(w).unwrap()).join(" "),
                             pmi));
        }
    }
}

extern crate getopts;
extern crate mi;
extern crate stdinout;

use std::env::args;
use std::io::{BufRead, BufWriter, Write};
use std::process;

use getopts::Options;
use stdinout::*;

use mi::*;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] EMBEDDINGS", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");

    let matches = or_exit(opts.parse(&args[1..]));

    if matches.opt_present("h") {
        print_usage(&program, opts);
        process::exit(1)
    }

    if matches.free.len() > 2 {
        print_usage(&program, opts);
        process::exit(1)
    }

    let input = Input::from(matches.free.get(0));
    let reader = or_exit(input.buf_read());

    let output = Output::from(matches.free.get(1));
    let mut writer = BufWriter::new(or_exit(output.write()));

    let mut collector = Collector::new();

    for line in reader.lines() {
        let line = or_exit(line);

        let parts: Vec<_> = line.trim().split_whitespace().collect();
        if parts.len() != 2 {
            stderr!("Line without two columns: {}", line);
            process::exit(1);
        }

        collector.count(parts[0].to_owned(), parts[1].to_owned());
    }

    collector.filter_freq(2);

    for ((w1, w2), pmi) in collector.iter(MutualInformation::NPMI, Smoothing::Add(2)) {
        or_exit(writeln!(writer, "{} {} {}", w1, w2, pmi));
    }
}

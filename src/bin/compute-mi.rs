extern crate getopts;
#[macro_use]
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
    opts.optopt("f", "freq", "minimum frequency cut-off", "N");
    opts.optflag("h", "help", "print this help menu");

    let matches = or_exit(opts.parse(&args[1..]));

    if matches.opt_present("h") {
        print_usage(&program, opts);
        process::exit(1)
    }

    let cutoff = matches.opt_str("f").map(|v| or_exit(v.parse())).unwrap_or(1);

    if matches.free.len() > 2 {
        print_usage(&program, opts);
        process::exit(1)
    }

    let input = Input::from(matches.free.get(0));
    let reader = or_exit(input.buf_read());

    let output = Output::from(matches.free.get(1));
    let mut writer = BufWriter::new(or_exit(output.write()));

    let mut word_map = WordMap::new();
    let mut collector = Collector::new();

    for line in reader.lines() {
        let line = or_exit(line);

        let parts: Vec<_> = line.trim().split_whitespace().map(|w| word_map.number(w)).collect();

        if parts.len() != 3 {
            stderr!("Line without three columns: {}", line);
            process::exit(1);
        }

        let triple = [parts[0].to_owned(), parts[1].to_owned(), parts[2].to_owned()];

        collector.count(triple);
    }

    for (triple, freq, pmi) in collector.iter(MutualInformation::NSC) {
        if freq >= cutoff {
            or_exit(writeln!(writer,
                             "{} {} {} {}",
                             word_map.word(triple[0]).unwrap(),
                             word_map.word(triple[1]).unwrap(),
                             word_map.word(triple[2]).unwrap(),
                             pmi));
        }
    }
}

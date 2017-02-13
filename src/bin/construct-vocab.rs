extern crate flate2;
extern crate fst;
extern crate getopts;
extern crate mi;
extern crate stdinout;

use std::collections::BTreeSet;
use std::collections::HashSet;
use std::env::args;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::process;

use flate2::read::GzDecoder;
use fst::MapBuilder;
use getopts::Options;

use mi::*;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] INPUT VOCAB_OUT", program);
    print!("{}", opts.usage(&brief));
}

fn open_reader(filename: &str, gzip: bool) -> io::Result<Box<Read>> {
    let f = File::open(filename)?;

    let r = if gzip {
        Box::new(BufReader::new(GzDecoder::new(f)?)) as Box<Read>
    } else {
        Box::new(f)
    };

    Ok(r)
}

fn main() {
    let args: Vec<String> = args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("z", "gzip", "read gzipped input");

    let matches = or_exit(opts.parse(&args[1..]));

    if matches.opt_present("h") {
        print_usage(&program, opts);
        process::exit(1)
    }

    if matches.free.len() != 2 {
        print_usage(&program, opts);
        process::exit(1)
    }

    let reader = BufReader::new(or_exit(open_reader(&matches.free[0], matches.opt_present("z"))));
    let writer = BufWriter::new(or_exit(File::create(&matches.free[1])));

    let mut vocab = HashSet::new();

    for line in reader.lines() {
        let line = or_exit(line);

        let parts: Vec<_> = line.trim().split_whitespace().collect();
        if parts.len() != 3 {
            stderr!("Line without two columns: {}", line);
            process::exit(1);
        }

        for part in parts {
            vocab.insert(part.as_bytes().to_owned());
        }
    }

    let ordered_vocab: BTreeSet<_> = vocab.into_iter().collect();

    let mut builder = or_exit(MapBuilder::new(writer));
    or_exit(builder.extend_iter(ordered_vocab.iter().enumerate().map(|(a, b)| (b, a as u64))));
    or_exit(builder.finish());
}

# mi

## Introduction

This project provides a small utility, `compute-mi` to compute mutual
information between two or more random variables. This utility is
optimized for:

- **Memory use**: since mutual information is typically computed over
  large data sets, we try to keep memory use small. This is done in
  two ways: (1) strings from the input are internally represented as
  numbers; (2) counts of joint events are stored in a hash map where
  the keys are tuples (fixed-length arrays), avoiding the overhead
  of regular dynamic arrays.

- **Flexibility**: any file that has whitespace-separated columns can
  be used as the input. You can specify which columns mutual information
  should be computed for.

## Measures

Currently, the following mutual information measures are supported:

- **Specific correlation**: this is a generalization of mutual information
  for multiple variables (Van de Cruys, 2011).
- **Normalized specific correlation**: This is our extension of normalized
  PMI (Bouma, 2009) to specific correlation.
- **Positive mutual information:** all measures can be used in the positive
  variant, where negative mutual information is set to 0.

## Installation

To compile and install *mi*, you need the Rust compiler and the Cargo
package manager. Using [rustup.rs](https://www.rustup.rs) is the recommended
method for installing both. After installing Rust, you can compile and
install this package by simply running

~~~{.bash}
cargo install
~~~

in the main directory. Afterwards, `compute-mi` should be in your path.

## Usage

The basic usage of `compute-mi` is:

~~~
target/release/compute-mi [options] VARS [INPUT] [OUTPUT]
~~~

where `VARS` is a comma-delimited list of columns that should be used as
variables. The input and output files are optional. `stdin`/`stdout` are
used when they are not provided.

This is an example run using two variables:

~~~{.bash}
zcat < taz-obja-words.gz | target/release/compute-mi -m nsc 1,2
~~~

## Caveats

### Double counting of events

The events in each joint event (line in the input) are considered to be
unique occurrences.  E.g. if we are computing specific correlations on
triples of head-prep-prep_obj:

~~~
walk on pavement
walk in city
~~~

*walk* will be counted twice, although it may actually be a single token
which has the two dependents *on* and *in*.
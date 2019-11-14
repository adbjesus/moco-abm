use moco_abm::model2d::{LinearSegment2D, Model2D, Scalar};

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};

fn main() {
    if let Err(e) = execute() {
        eprintln!("Error: {}\n", e);
        eprintln!("{}", usage(&env::args().next().unwrap()));
        std::process::exit(1);
    }
}

fn usage(program: &str) -> String {
    format!(
        "\
Usage: 
  {} n d r_1 r_2 ... r_d [file]

Where:
  n      Number of points to return. Must be greater than 0.
  d      Number of dimensions. Currently only 2 is supported.
  r_i    Value of the reference point on the i-th coordinate.
  file   Optional argument for segments, otherwise read from stdin.\
",
        program
    )
}

fn execute() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();

    let _ = args.next();

    let n = args
        .next()
        .ok_or("missing argument `n`")?
        .parse::<usize>()?;

    let d = args
        .next()
        .ok_or("missing argument `d`")?
        .parse::<usize>()?;

    let mut r: Vec<f64> = Vec::with_capacity(d);
    for i in 0..d {
        r.push(
            args.next()
                .ok_or(format!("missing argument `r_{}`", i + 1))?
                .parse::<f64>()?,
        )
    }

    let s = match args.next() {
        Some(f) => read_segments(File::open(f)?)?,
        None => read_segments(io::stdin())?,
    };

    let mut m = Model2D::new(s, r)?;

    println!("index\thv_contribution\thv_current\thv_relative\tpoint");
    for i in 1..(n + 1) {
        let (point, hv_contribution, hv_current, hv_relative) =
            match m.get_next_point() {
                Some(r) => r,
                None => break,
            };

        println!(
            "{}\t{:.12}\t{:.12}\t{:.12}\t{:.12},{:.12}",
            i, hv_contribution, hv_current, hv_relative, point[0], point[1]
        );
    }

    Ok(())
}

fn read_segments<T: Scalar>(
    mut r: impl Read,
) -> Result<Vec<LinearSegment2D<T>>, Box<dyn Error>> {
    let mut buffer = String::new();
    r.read_to_string(&mut buffer)?;

    let mut v = Vec::new();
    let mut iter = buffer.split_whitespace();
    loop {
        let start = [
            match iter.next() {
                Some(s) => T::from_str_radix(s, 10)
                    .ok()
                    .ok_or("failed to parse coordinate data")?,
                None => break,
            },
            match iter.next() {
                Some(s) => T::from_str_radix(s, 10)
                    .ok()
                    .ok_or("failed to parse coordinate data")?,
                None => break,
            },
        ];
        let end = [
            match iter.next() {
                Some(s) => T::from_str_radix(s, 10)
                    .ok()
                    .ok_or("failed to parse coordinate data")?,
                None => break,
            },
            match iter.next() {
                Some(s) => T::from_str_radix(s, 10)
                    .ok()
                    .ok_or("failed to parse coordinate data")?,
                None => break,
            },
        ];
        v.push(LinearSegment2D::new(start, end));
    }

    Ok(v)
}

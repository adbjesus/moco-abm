use moco_abm::model2d::{generate_segments, LinearSegment2D, Model2D, Scalar};

use std::env;
use std::error::Error;
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
  {} k m r_1 r_2 ... r_d [n d]

Where:
  k      Number of points to return. Must be greater than 0.
  m      Number of dimensions. Currently only 2 is supported.
  r_i    Value of the reference point on the i-th coordinate.
  n,d    Optional arguments to generate 'n' linear segments for the superellipse
         curve approximation of parameter 'd'. If these are not given, we expect
         to read a list of segments from stdin (see README for format).\
",
        program
    )
}

fn execute() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();

    let _ = args.next();

    let k = args
        .next()
        .ok_or("missing argument `k`")?
        .parse::<usize>()?;

    let m = args
        .next()
        .ok_or("missing argument `m`")?
        .parse::<usize>()?;

    let mut r: Vec<f64> = Vec::with_capacity(m);
    for i in 0..m {
        r.push(
            args.next()
                .ok_or(format!("missing argument `r_{}`", i + 1))?
                .parse::<f64>()?,
        )
    }

    let s = match args.next() {
        Some(v) => {
            let n = v.parse::<usize>()?;
            let d =
                args.next().ok_or("missing argument 'd'")?.parse::<f64>()?;
            generate_segments(n, d)?
        }
        None => read_segments(io::stdin())?,
    };

    let mut m = Model2D::new(s, [r[0], r[1]])?;
    let points = m.solve(k);

    println!("hv_contribution\thv_current\thv_relative\tpoint");
    for (point, hv_contribution, hv_current, hv_relative) in points {
        println!(
            "{:.12}\t{:.12}\t{:.12}\t{:.12},{:.12}",
            hv_contribution, hv_current, hv_relative, point[0], point[1]
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
        v.push(LinearSegment2D::new(start, end)?);
    }

    Ok(v)
}

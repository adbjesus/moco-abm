#[macro_use]
extern crate clap;

extern crate moco_abm;

use clap::{AppSettings, Arg};
use moco_abm::model2d::{LinearSegment2D, Model2D};

use std::error::Error;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

fn main() {
    if let Err(e) = execute() {
        eprintln!("Error: {}", e);
    }
}

fn execute() -> Result<(), Box<dyn Error>> {
    let matches = app_from_crate!()
        .setting(AppSettings::AllowNegativeNumbers)
        .arg(
            Arg::with_name("num")
                .help("number of points to retrieve")
                .short("n")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("file")
                .help("file with piecewise approximation definition (stdin is used if not set)")
                .short("f")
                .takes_value(true),
        )
        .get_matches();

    let n = validate_num(parse_num(matches.value_of("num").unwrap())?)?;
    let f = parse_file(matches.value_of("file"))?;
    let s = match f {
        Some(f) => read_segments(f),
        None => read_segments(io::stdin()),
    }?;

    let mut m = Model2D::new(s)?;

    println!("index\thv_contribution\thv_current\thv_relative\tpoint");
    for i in 1..(n + 1) {
        let (point, hv_contribution, hv_current, hv_relative) =
            match m.get_next_point() {
                Some(r) => r,
                None => break,
            };

        println!(
            "{}\t{}\t{}\t{}\t{},{}",
            i, hv_contribution, hv_current, hv_relative, point[0], point[1]
        );
    }

    Ok(())
}

fn parse_num(s: &str) -> Result<usize, impl Error> {
    s.parse::<usize>()
}

fn parse_file(s: Option<&str>) -> Result<Option<File>, String> {
    match s {
        Some(s) => {
            let p = PathBuf::from(&s);
            if !p.exists() {
                return Err(format!("<file> `{}` does not exist", s));
            }
            if !p.is_file() {
                return Err(format!("<file> `{}` is not a file", s));
            }
            match File::open(s) {
                Ok(f) => Ok(Some(f)),
                Err(e) => {
                    Err(format!("<file> `{}` could not be opened - {}", s, e))
                }
            }
        }
        None => Ok(None),
    }
}

fn read_segments(
    mut r: impl Read,
) -> Result<Vec<LinearSegment2D>, Box<dyn Error>> {
    let mut buffer = String::new();
    r.read_to_string(&mut buffer)?;

    let mut v = Vec::new();
    let mut iter = buffer.split_whitespace();
    loop {
        let start = [
            match iter.next() {
                Some(p) => p.parse::<f64>()?,
                None => break,
            },
            iter.next()
                .ok_or("missing coordinate data")?
                .parse::<f64>()?,
        ];
        let end = [
            iter.next()
                .ok_or("missing coordinate data")?
                .parse::<f64>()?,
            iter.next()
                .ok_or("missing coordinate data")?
                .parse::<f64>()?,
        ];
        v.push(LinearSegment2D::new(start, end));
    }

    Ok(v)
}

fn validate_num(n: usize) -> Result<usize, &'static str> {
    if n < 1 {
        return Err("-n <num> option must be a positive integer");
    }
    Ok(n)
}

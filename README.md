# Multi-Objective Combinatorial Optimization - Anytime Behavior Model (moco-abm)

This software provides both a binary and a library (see note) for an anytime behavior model of multi-objective combinatorial optimization algorithms that, at each iteration, collect an efficient solution that maximizes the hypervolume contribution. It is assumed that all objective functions are to be maximized.

**Note**: the current software is intended to be used as a binary for now. The library API is not yet properly defined and no documentation is provided for now.

## Binary

Install the latest binary using `cargo` with:

```sh
cargo install moco-abm-bin
```

or compile from source with:

```sh
cargo build --release
```

### Usage

```
USAGE:
    moco-abm [OPTIONS] -n <num>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f <file>        file with piecewise approximation definition (stdin is used if not set)
    -n <num>         number of points to retrieve
```

The input file should contain at least one segment in the following format

```
u1 u2 v1 v2
```

where $(u_1, u_2)$ and $(v_1, v_2)$ denote the coordinates of the linear segment endpoints. Multiple segments, and coordinates within the segments, can be separated by any whitespace.

**Note**: Points in the segments must be provided such that `v1 > u1` and `v2 < u2`. Moreover, when multiple segments are provided, e.g.:

```
u1 u2 v1 v2
p1 p2 q1 q2
```

it is required that `p1 >= v1` and that `p2 <= v2`.

Example of a valid segments list file:

```
0.0 1.0 0.7 0.7
0.7 0.7 1.0 0.0
```

### Output

The output is returned to `stdout` and consists of a `.tsv` with the following fields

| field           | description                                  |
|:----------------|:---------------------------------------------|
| index           | index of the current point (starts at 1)     |
| hv_contribution | hypervolume contribution of this point       |
| hv_current      | hypervolume of all returned points up to now |
| hv_relative     | current_hv relative to maximal hypervolume   |
| point           | comma separated coordinates of the point     |


## Library

Add this to your `Cargo.toml`:

```
[dependencies]
moco_abm = "0.1"
```

and this to your crate root:

```rust
extern crate moco_abm;
```

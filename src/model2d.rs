use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::Debug;

use num_traits::real::Real;

use crate::error::{Error, ErrorKind};

pub trait Scalar: Real + Debug + PartialOrd {
    fn two() -> Self {
        Self::one() + Self::one()
    }

    fn eq_epsilon(&self, rhs: Self) -> bool {
        Self::abs(*self - rhs) < Self::epsilon()
    }

    fn ge_epsilon(&self, rhs: Self) -> bool {
        *self > rhs || self.eq_epsilon(rhs)
    }

    fn le_epsilon(&self, rhs: Self) -> bool {
        *self < rhs || self.eq_epsilon(rhs)
    }
}

impl<T: Real + Debug + PartialOrd> Scalar for T {}

pub type Point2D<T> = [T; 2];

#[derive(Debug)]
pub struct Model2D<T: Scalar> {
    regions: BinaryHeap<Region2D<T>>,
    current_hv: T,
    max_hv: T,
}

#[derive(Debug)]
struct Region2D<T: Scalar> {
    chain: Vec<LinearSegment2D<T>>,
    reference: Point2D<T>,
    best_hv: T,
    best_point: Point2D<T>,
}

#[derive(Debug)]
pub struct LinearSegment2D<T: Scalar> {
    start: Point2D<T>,
    end: Point2D<T>,
}

impl<T: Scalar> Model2D<T> {
    pub fn new(
        s: Vec<LinearSegment2D<T>>,
        r: Point2D<T>,
    ) -> Result<Model2D<T>, Error> {
        validate_segments(&s)?;

        let r = [r[0], r[1]];
        let mut regions = BinaryHeap::new();
        let mut hv = T::zero();
        if let Some(reg) = Region2D::new(s, r) {
            hv = hv + calculate_segments_hv(&reg.chain, &r);
            regions.push(reg);
        }

        Ok(Model2D {
            regions: regions,
            current_hv: T::zero(),
            max_hv: hv,
        })
    }

    pub fn get_next_point(&mut self) -> Option<(Point2D<T>, T, T, T)> {
        match self.regions.pop() {
            Some(r) => {
                self.current_hv = self.current_hv + r.best_hv;

                let best_point = r.best_point;
                let best_hv = r.best_hv;

                let (region_above, region_below) = r.split_at_best();
                if let Some(ra) = region_above {
                    self.regions.push(ra);
                }
                if let Some(rb) = region_below {
                    self.regions.push(rb);
                }

                Some((
                    best_point,
                    best_hv,
                    self.current_hv,
                    self.current_hv / self.max_hv,
                ))
            }
            None => None,
        }
    }

    pub fn solve(&mut self, n: usize) -> Vec<(Point2D<T>, T, T, T)> {
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            match self.get_next_point() {
                Some(p) => v.push(p),
                None => break,
            }
        }
        return v;
    }
}

impl<T: Scalar> Region2D<T> {
    fn new(segs: Vec<LinearSegment2D<T>>, r: Point2D<T>) -> Option<Self> {
        if segs.len() == 0 {
            return None;
        }

        let mut best_hv = T::zero();
        let mut best_point = [T::zero(), T::zero()];

        let mut chain = Vec::new();

        for mut s in segs.into_iter() {
            if let Some((hv, p)) = s.best_hv(r) {
                if hv > best_hv {
                    best_hv = hv;
                    best_point = p;
                }
                chain.push(s);
            }
        }

        return if chain.is_empty() {
            None
        } else {
            Some(Self {
                chain: chain,
                reference: r,
                best_hv,
                best_point,
            })
        };
    }

    fn split_at_best(self) -> (Option<Region2D<T>>, Option<Region2D<T>>) {
        if self.chain.is_empty() {
            return (None, None);
        }

        if self.best_hv.eq_epsilon(T::zero()) {
            return (None, None);
        }

        let mut segs_above = Vec::new();
        let mut segs_below = Vec::new();

        for s in self.chain.into_iter() {
            if s.end[0].le_epsilon(self.best_point[0]) {
                segs_above.push(s);
            } else if s.start[0].ge_epsilon(self.best_point[0]) {
                segs_below.push(s);
            } else {
                segs_above.push(
                    LinearSegment2D::new(s.start, self.best_point).unwrap(),
                );
                segs_below.push(
                    LinearSegment2D::new(self.best_point, s.end).unwrap(),
                );
            }
        }

        let region_above =
            Region2D::new(segs_above, [self.reference[0], self.best_point[1]]);

        let region_below =
            Region2D::new(segs_below, [self.best_point[0], self.reference[1]]);

        (region_above, region_below)
    }
}

impl<T: Scalar> Ord for Region2D<T> {
    fn cmp(&self, other: &Region2D<T>) -> Ordering {
        match self.partial_cmp(other) {
            Some(o) => o,
            None => {
                panic!("Unexpected behavior: invalid comparison of scalars")
            }
        }
    }
}

impl<T: Scalar> PartialOrd for Region2D<T> {
    fn partial_cmp(&self, other: &Region2D<T>) -> Option<Ordering> {
        self.best_hv.partial_cmp(&other.best_hv)
    }
}

impl<T: Scalar> Eq for Region2D<T> {}

impl<T: Scalar> PartialEq for Region2D<T> {
    fn eq(&self, other: &Region2D<T>) -> bool {
        self.best_hv.eq(&other.best_hv)
    }
}

impl<T: Scalar> LinearSegment2D<T> {
    pub fn new(
        start: Point2D<T>,
        end: Point2D<T>,
    ) -> Result<Self, &'static str> {
        if start[0] < end[0] && start[1] > end[1] {
            Ok(Self {
                start: start,
                end: end,
            })
        } else if start[0] > end[0] && start[1] < end[1] {
            Ok(Self {
                start: end,
                end: start,
            })
        } else {
            Err("LinearSegment2D endpoints must be non-dominated")
        }
    }

    fn best_hv(&mut self, r: Point2D<T>) -> Option<(T, Point2D<T>)> {
        /* Calculate line equation */
        let m = (self.end[1] - self.start[1]) / (self.end[0] - self.start[0]);
        let b = self.end[1] - m * self.end[0];

        // Point where line meets the reference axis
        let p = [(r[1] - b) / m, m * r[0] + b];
        if p[0] < r[0] || p[1] < r[1] {
            return None;
        }

        // Truncate end point to region
        if p[0] < self.end[0] {
            self.end[1] = r[1];
            self.end[0] = (self.end[1] - b) / m;
        }
        // Truncate start point to region
        if p[1] < self.start[1] {
            self.start[0] = r[0];
            self.start[1] = m * self.start[0] + b;
        }

        /* Calculate catheti */
        let c = [(r[1] - b) / m - r[0], m * r[0] + b - r[1]];

        /* Find optimal point in the hypothenuse */
        let mut u = [r[0] + c[0] / T::two(), r[1] + c[1] / T::two()];

        /* Change optimal point if outside of the segment */
        if u[0].le_epsilon(self.start[0]) {
            u = self.start;
        } else if u[0].ge_epsilon(self.end[0]) {
            u = self.end;
        }

        let hv = (u[0] - r[0]) * (u[1] - r[1]);

        if hv < T::zero() {
            return None;
        } else {
            return Some((hv, u));
        }
    }
}

fn validate_segments<T: Scalar>(
    s: &Vec<LinearSegment2D<T>>,
) -> Result<(), Error> {
    for i in 1..s.len() {
        if !s[i].start[0].ge_epsilon(s[i - 1].end[0])
            || !s[i].start[1].le_epsilon(s[i - 1].end[1])
        {
            return Err(Error::new(ErrorKind::UnsortedSegments));
        }
    }

    Ok(())
}

fn calculate_segments_hv<T: Scalar>(
    s: &Vec<LinearSegment2D<T>>,
    r: &Point2D<T>,
) -> T {
    let mut hv = T::zero();
    let mut r0 = r[0];
    let r1 = r[1];
    for i in s {
        hv = hv + (i.start[0] - r0) * (i.start[1] - r1);
        r0 = i.start[0];
        hv = hv + (i.end[0] - r0) * (i.end[1] - r1);
        hv = hv + (i.end[0] - i.start[0]) * (i.start[1] - i.end[1]) / T::two();
        r0 = i.end[0];
    }
    hv
}

pub fn generate_segments<T: Scalar>(
    n: usize,
    d: f64,
) -> Result<Vec<LinearSegment2D<T>>, &'static str> {
    if n < 1 || d <= 0f64 {
        return Err("Invalid values of 'n' or 'd' to generate segments");
    }
    let step = std::f64::consts::PI / 2f64 / (n as f64);
    let pi2 = std::f64::consts::PI / 2f64;
    let pow = 2f64 / d;
    let mut segs = Vec::new();
    for i in 0..n {
        let theta_s = (step * (i as f64)).max(0f64).min(pi2);
        let theta_e = (step * ((i + 1) as f64)).max(0f64).min(pi2);
        let y0_s = T::from(theta_s.sin().powf(pow)).unwrap();
        let y1_s = T::from(theta_s.cos().powf(pow)).unwrap();
        let y0_e = T::from(theta_e.sin().powf(pow)).unwrap();
        let y1_e = T::from(theta_e.cos().powf(pow)).unwrap();
        segs.push(LinearSegment2D::new([y0_s, y1_s], [y0_e, y1_e]).unwrap());
    }
    return Ok(segs);
}

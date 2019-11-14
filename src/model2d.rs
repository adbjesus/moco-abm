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

impl<T: Real + Debug> Scalar for T {}

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
    best_ind: usize,
    best_point: Point2D<T>,
    best_location: PointLocation2D,
}

#[derive(Debug)]
pub struct LinearSegment2D<T: Scalar> {
    start: Point2D<T>,
    end: Point2D<T>,
}

#[derive(Debug)]
enum PointLocation2D {
    Start,
    Middle,
    End,
}

impl<T: Scalar> Model2D<T> {
    pub fn new(
        s: Vec<LinearSegment2D<T>>,
        r: Vec<T>,
    ) -> Result<Model2D<T>, Error> {
        if r.len() != 2 {
            return Err(Error::new(ErrorKind::WrongDimensions));
        }

        validate_segments(&s)?;
        // println!("{:#?}", s);

        let r = [r[0], r[1]];
        let mut regions = BinaryHeap::new();
        let mut hv = T::zero();
        if let Some(reg) = Region2D::new(s, r) {
            hv = hv + calculate_segments_hv(&reg.chain, &r);
            // println!("{:#?}", reg);
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
}

impl<T: Scalar> Region2D<T> {
    fn new(mut s: Vec<LinearSegment2D<T>>, r: Point2D<T>) -> Option<Self> {
        if s.len() == 0 {
            return None;
        }

        let mut best_hv = T::zero();
        let mut best_ind = 0;
        let mut best_point = [T::zero(), T::zero()];
        let mut best_location = PointLocation2D::Start;

        let mut remove = 0usize;
        for i in 0..s.len() {
            if let Some((tmp_hv, tmp_point, tmp_location)) =
                s[i - remove].best_hv(r)
            {
                if tmp_hv > best_hv {
                    best_hv = tmp_hv;
                    best_ind = i;
                    best_point = tmp_point;
                    best_location = tmp_location;
                }
            } else {
                s.remove(i - remove);
                remove += 1;
            }
        }

        return if s.is_empty() {
            None
        } else {
            Some(Self {
                chain: s,
                reference: r,
                best_hv,
                best_ind,
                best_point,
                best_location,
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

        let mut segments_above = Vec::with_capacity(self.best_ind + 1);
        let mut segments_below =
            Vec::with_capacity(self.chain.len() - self.best_ind + 1);

        let mut chain_iter = self.chain.into_iter();
        for _ in 0..self.best_ind {
            segments_above.push(chain_iter.next().unwrap());
        }

        let s = chain_iter.next().unwrap();
        match &self.best_location {
            PointLocation2D::Start => segments_below.push(s),
            PointLocation2D::End => segments_above.push(s),
            PointLocation2D::Middle => {
                segments_above
                    .push(LinearSegment2D::new(s.start, self.best_point));
                segments_below
                    .push(LinearSegment2D::new(self.best_point, s.end));
            }
        }

        for s in chain_iter {
            segments_below.push(s);
        }

        let region_above = Region2D::new(
            segments_above,
            [self.reference[0], self.best_point[1]],
        );

        let region_below = Region2D::new(
            segments_below,
            [self.best_point[0], self.reference[1]],
        );

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
    pub fn new(start: Point2D<T>, end: Point2D<T>) -> Self {
        Self { start, end }
    }

    fn best_hv(
        &mut self,
        r: Point2D<T>,
    ) -> Option<(T, Point2D<T>, PointLocation2D)> {
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
        let u = [r[0] + c[0] / T::two(), r[1] + c[1] / T::two()];

        /* If point before start, set start */
        if u[0].le_epsilon(self.start[0]) {
            return Some((
                (self.start[0] - r[0]) * (self.start[1] - r[1]),
                [self.start[0], self.start[1]],
                PointLocation2D::Start,
            ));
        }

        /* If point before start, set start */
        if u[0].ge_epsilon(self.end[0]) {
            return Some((
                (self.end[0] - r[0]) * (self.end[1] - r[1]),
                [self.end[0], self.end[1]],
                PointLocation2D::End,
            ));
        }

        /* Else set middle */
        return Some((
            (u[0] - r[0]) * (u[1] - r[1]),
            [u[0], u[1]],
            PointLocation2D::Middle,
        ));
    }
}

fn validate_segments<T: Scalar>(
    s: &Vec<LinearSegment2D<T>>,
) -> Result<(), Error> {
    if s.len() == 0 {
        return Err(Error::with_message(
            ErrorKind::EmptyApproximation,
            String::from("at least one segment is required"),
        ));
    }

    for i in s {
        if i.start[0].ge_epsilon(i.end[0]) || i.start[1].le_epsilon(i.end[1]) {
            println!("{:?}", i);
            println!("{:?}", i.start[0].ge_epsilon(i.end[0]));
            println!("{:?}", i.start[0] > i.end[0]);
            println!("{:?}", i.start[0].eq_epsilon(i.end[0]));
            println!("{:?}", i.start[1].le_epsilon(i.end[1]));
            return Err(Error::with_message(
                ErrorKind::UnsortedSegment,
                String::from(
                    "Linearsegments needs to be sorted such that start[0] < end[0] && start[1] > end[1]"
                ),
            ));
        }
    }

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

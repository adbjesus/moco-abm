use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::error::{Error, ErrorKind};

pub type Point2D = [f64; 2];

#[derive(Debug)]
pub struct Model2D {
    regions: BinaryHeap<Region2D>,
    current_hv: f64,
    max_hv: f64,
}

#[derive(Debug)]
struct Region2D {
    chain: Vec<LinearSegment2D>,
    reference: Point2D,
    best_hv: f64,
    best_ind: usize,
    best_point: Point2D,
    best_location: PointLocation2D,
}

#[derive(Debug)]
pub struct LinearSegment2D {
    start: Point2D,
    end: Point2D,
}

#[derive(Debug)]
enum PointLocation2D {
    Start,
    Middle,
    End,
}

impl Model2D {
    pub fn new(s: Vec<LinearSegment2D>) -> Result<Model2D, Error> {
        validate_segments(&s)?;

        let r = calculate_reference(&s);
        let h = calculate_segments_hv(&s, r);

        let mut regions = BinaryHeap::new();
        regions.push(Region2D::new(s, r)?);

        Ok(Model2D {
            regions: regions,
            current_hv: 0.,
            max_hv: h,
        })
    }

    pub fn get_next_point(&mut self) -> Option<(Point2D, f64, f64, f64)> {
        match self.regions.pop() {
            Some(r) => {
                self.current_hv += r.best_hv;

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

impl Region2D {
    fn new(s: Vec<LinearSegment2D>, r: Point2D) -> Result<Region2D, Error> {
        if s.len() == 0 {
            return Err(Error::new(ErrorKind::EmptyRegion));
        }

        let mut best_hv = 0.;
        let mut best_ind = 0;
        let mut best_point = [0., 0.];
        let mut best_location = PointLocation2D::Start;

        for i in 0..s.len() {
            let (tmp_hv, tmp_point, tmp_location) = s[i].best_hv(r);

            if tmp_hv > best_hv {
                best_hv = tmp_hv;
                best_ind = i;
                best_point = tmp_point;
                best_location = tmp_location;
            }
        }

        Ok(Region2D {
            chain: s,
            reference: r,
            best_hv,
            best_ind,
            best_point,
            best_location,
        })
    }

    fn split_at_best(self) -> (Option<Region2D>, Option<Region2D>) {
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
        )
        .ok();

        let region_below = Region2D::new(
            segments_below,
            [self.best_point[0], self.reference[1]],
        )
        .ok();

        (region_above, region_below)
    }
}

impl Ord for Region2D {
    fn cmp(&self, other: &Region2D) -> Ordering {
        match self.partial_cmp(other) {
            Some(o) => o,
            None => panic!(
                "Unexpected behavior: invalid comparison of f64 - {} , {}",
                self.best_hv, other.best_hv
            ),
        }
    }
}

impl PartialOrd for Region2D {
    fn partial_cmp(&self, other: &Region2D) -> Option<Ordering> {
        self.best_hv.partial_cmp(&other.best_hv)
    }
}

impl Eq for Region2D {}

impl PartialEq for Region2D {
    fn eq(&self, other: &Region2D) -> bool {
        self.best_hv.eq(&other.best_hv)
    }
}

impl LinearSegment2D {
    pub fn new(start: Point2D, end: Point2D) -> LinearSegment2D {
        LinearSegment2D { start, end }
    }

    fn best_hv(&self, r: Point2D) -> (f64, Point2D, PointLocation2D) {
        /* Adjust to reference */
        let s = [self.start[0] - r[0], self.start[1] - r[1]];
        let e = [self.end[0] - r[0], self.end[1] - r[1]];

        /* Calculate line equation */
        let m = (e[1] - s[1]) / (e[0] - s[0]);
        let b = s[1] - m * s[0];

        /* Calculate triangle catheti */
        let c = [-b / m, b];

        /* Find optimal point in the hypothenuse */
        let u = [c[0] / 2., c[1] / 2.];

        /* If point `u` is within the segment return it */
        if u[0] > s[0] && u[0] < e[0] {
            return (
                u[0] * u[1],
                [u[0] + r[0], u[1] + r[1]],
                PointLocation2D::Middle,
            );
        }

        /* Otherwise find closest point on */
        if u[0] <= s[0] {
            return (
                s[0] * s[1],
                [s[0] + r[0], s[1] + r[1]],
                PointLocation2D::Start,
            );
        }

        return (
            e[0] * e[1],
            [e[0] + r[0], e[1] + r[1]],
            PointLocation2D::End,
        );
    }
}

fn validate_segments(s: &Vec<LinearSegment2D>) -> Result<(), Error> {
    if s.len() == 0 {
        return Err(Error::with_message(
            ErrorKind::EmptyApproximation,
            String::from("at least one segment is required"),
        ));
    }

    for i in s {
        if i.start[0] >= i.end[0] || i.start[1] <= i.end[1] {
            return Err(Error::with_message(
                ErrorKind::UnsortedSegment,
                format!(
                    "{:?} needs to be sorted such that start[0] < end[0] && start[1] > end[1]", i
                ),
            ));
        }
    }

    for i in 1..s.len() {
        if s[i].start[0] < s[i - 1].end[0] {
            return Err(Error::new(ErrorKind::UnsortedSegments));
        }
    }

    Ok(())
}

fn calculate_reference(s: &Vec<LinearSegment2D>) -> Point2D {
    [s[0].start[0], s[s.len() - 1].end[1]]
}

fn calculate_segments_hv(s: &Vec<LinearSegment2D>, r: Point2D) -> f64 {
    let mut hv = 0.;
    let mut r0 = r[0];
    let r1 = r[1];
    for i in s {
        hv += (i.start[0] - r0) * (i.start[1] - r1);
        r0 = i.start[0];
        hv += (i.end[0] - r0) * (i.end[1] - r1);
        hv += (i.end[0] - i.start[0]) * (i.start[1] - i.end[1]) / 2.;
        r0 = i.end[0];
    }
    hv
}

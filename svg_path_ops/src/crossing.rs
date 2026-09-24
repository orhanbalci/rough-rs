//! Telling a crossing from a touch, for [`Intersection::crossing`].
//!
//! [`Intersection::crossing`]: crate::Intersection::crossing

use kurbo::{Point, Vec2};

use crate::measure::{PathMeasure, Piece};
use crate::Location;

/// How far along the paths the probes go at most, as a fraction of the
/// size of the two paths.
const PROBE: f64 = 1e-3;

/// Lengths closer than this, as a fraction of the size, count as the same.
const SAME_LENGTH: f64 = 1e-9;

/// One path's side of an intersection: the subpath it lies on, and how far
/// its probes may go from it.
struct Side<'a> {
    /// The drawing pieces of the subpath
    pieces: Vec<&'a Piece>,
    closed: bool,
    length: f64,
}

impl<'a> Side<'a> {
    fn new(measure: &'a PathMeasure, at: &Location) -> Option<Self> {
        let subpath = measure
            .pieces
            .iter()
            .find(|piece| piece.index == at.position.index)?
            .subpath;
        let all: Vec<&Piece> = measure
            .pieces
            .iter()
            .filter(|p| p.subpath == subpath)
            .collect();
        let closed = all.last().is_some_and(|piece| piece.closes);
        let pieces = all.into_iter().filter(|piece| piece.length > 0.0).collect();
        Some(Side { pieces, closed, length: at.length })
    }

    fn start(&self) -> f64 {
        self.pieces[0].start
    }

    fn end(&self) -> f64 {
        let last = self.pieces[self.pieces.len() - 1];
        last.start + last.length
    }

    /// How far a probe may go from the point: not past an open end, and
    /// not more than a quarter of the segments around the point.
    fn reach(&self, size: f64) -> f64 {
        let near = size * SAME_LENGTH;
        let mut reach = f64::INFINITY;
        if !self.closed {
            reach = reach
                .min(self.length - self.start())
                .min(self.end() - self.length);
            if reach <= near {
                return 0.0;
            }
            reach /= 2.0;
        }
        for piece in &self.pieces {
            let (from, to) = (piece.start - near, piece.start + piece.length + near);
            if (from..=to).contains(&self.length) {
                reach = reach.min(piece.length / 4.0);
            }
        }
        // At a closed subpath's start, the last segment comes before it
        if self.closed && (self.length - self.start() <= near || self.end() - self.length <= near) {
            reach = reach.min(self.pieces[self.pieces.len() - 1].length / 4.0);
            reach = reach.min(self.pieces[0].length / 4.0);
        }
        reach
    }

    /// The point `offset` along the subpath from the intersection, going
    /// round a closed subpath.
    fn probe(&self, offset: f64) -> Point {
        let (start, end) = (self.start(), self.end());
        let mut target = self.length + offset;
        if self.closed {
            target = start + (target - start).rem_euclid(end - start);
        }
        let target = target.clamp(start, end);
        let piece = self
            .pieces
            .iter()
            .find(|piece| target <= piece.start + piece.length)
            .unwrap_or(&self.pieces[self.pieces.len() - 1]);
        let t = piece.shape.inv_arclen(target - piece.start, piece.length);
        piece.shape.eval(t)
    }
}

/// Whether `other` passes from one side of `this` to the other at `point`,
/// found at `here` on `this` and `there` on `other`, rather than touching
/// it. `gap_here` and `gap_there` are how far along each path the nearest
/// other intersection is.
///
/// A little way back and on along each path from the point, the probes of
/// `this` split the directions round the point into two sides; `other`
/// crosses when its two probes fall on different sides. Where the paths
/// run side by side, the probes see how they curve apart, so a line that
/// touches a circle does not cross it, and one through an inflection
/// does. A point at an open end of either path is not a crossing.
#[allow(clippy::too_many_arguments)]
pub(crate) fn is_crossing(
    this: &PathMeasure,
    other: &PathMeasure,
    point: Point,
    here: &Location,
    there: &Location,
    gap_here: f64,
    gap_there: f64,
    size: f64,
) -> bool {
    let (Some(a), Some(b)) = (Side::new(this, here), Side::new(other, there)) else {
        return false;
    };
    let probe = PROBE * size;
    let reach_a = a.reach(size).min(probe).min(gap_here / 3.0);
    let reach_b = b.reach(size).min(probe).min(gap_there / 3.0);
    if reach_a <= 0.0 || reach_b <= 0.0 {
        return false;
    }
    let (a1, a2) = (a.probe(-reach_a) - point, a.probe(reach_a) - point);
    let (b1, b2) = (b.probe(-reach_b) - point, b.probe(reach_b) - point);

    // Angles round the point, counterclockwise from `a1`
    let angle = |v: Vec2| (v.y.atan2(v.x) - a1.y.atan2(a1.x)).rem_euclid(std::f64::consts::TAU);
    let split = angle(a2);
    let (s1, s2) = (angle(b1), angle(b2));
    // A probe along one of `this`'s directions runs with it: no crossing
    let on_edge = |s: f64| s == 0.0 || s == split;
    if on_edge(s1) || on_edge(s2) {
        return false;
    }
    (s1 < split) != (s2 < split)
}

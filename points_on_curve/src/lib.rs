// This crate is entirely safe
#![forbid(unsafe_code)]
// Ensures that `pub` means published in the public API.
// This property is useful for reasoning about breaking API changes.
#![deny(unreachable_pub)]

use std::borrow::Borrow;
use std::cmp::{max_by, min_by, Ord};
use std::fmt::Display;
use std::ops::MulAssign;

use euclid::default::Point2D;
use euclid::point2;
use num_traits::Float;

/// computes distance squared from a point p to the line segment vw
pub fn distance_to_segment_squared<F, P>(p: P, v: P, w: P) -> F
where
    F: Float + PartialOrd + Display,
    P: Borrow<Point2D<F>>,
{
    let v_ = v.borrow();
    let w_ = w.borrow();
    let p_ = p.borrow();
    let l2 = v_.distance_to(*w_).powi(2);
    if l2 == F::zero() {
        p_.distance_to(*v_).powi(2)
    } else {
        let mut t = ((p_.x - v_.x) * (w_.x - v_.x) + (p_.y - v_.y) * (w_.y - v_.y)) / l2;
        t = max_by(
            F::zero(),
            min_by(F::one(), t, |a, b| {
                a.partial_cmp(b)
                    .expect(&format!("can not compare {} and {}", a, b))
            }),
            |a, b| {
                a.partial_cmp(b)
                    .expect(&format!("can not compare {} and {}", a, b))
            },
        );
        p_.distance_to(v_.lerp(*w_, t)).powi(2)
    }
}

/// Adapted from https://seant23.wordpress.com/2010/11/12/offset-bezier-curves/
pub fn flatness<F>(points: &[Point2D<F>], offset: usize) -> F
where
    F: Float + MulAssign,
{
    let p1 = points[offset + 0];
    let p2 = points[offset + 1];
    let p3 = points[offset + 2];
    let p4 = points[offset + 3];

    let mut ux = F::from(3).unwrap() * p2.x - F::from(2).unwrap() * p1.x - p4.x;
    ux *= ux;
    let mut uy = F::from(3).unwrap() * p2.y - F::from(2).unwrap() * p1.y - p4.y;
    uy *= uy;
    let mut vx = F::from(3).unwrap() * p3.x - F::from(2).unwrap() * p4.x - p1.x;
    vx *= vx;
    let mut vy = F::from(3).unwrap() * p3.y - F::from(2).unwrap() * p4.y - p1.y;
    vy *= vy;
    if ux < vx {
        ux = vx;
    }
    if uy < vy {
        uy = vy;
    }
    ux + uy
}

/// Ramer–Douglas–Peucker algorithm
/// https://en.wikipedia.org/wiki/Ramer%E2%80%93Douglas%E2%80%93Peucker_algorithm
pub fn simplify_points<F>(
    points: &[Point2D<F>],
    start: usize,
    end: usize,
    epsilon: F,
    new_points: &mut Vec<Point2D<F>>,
) -> Vec<Point2D<F>>
where
    F: Float + Display,
{
    let s = points[start];
    let e = points[end - 1];
    let mut max_dist_sq = F::zero();
    let mut max_ndx = 0;
    for i in start + 1..end - 1 {
        let distance_sq = distance_to_segment_squared(points[i], s, e);
        if distance_sq > max_dist_sq {
            max_dist_sq = distance_sq;
            max_ndx = i;
        }
    }

    if max_dist_sq.sqrt() > epsilon {
        simplify_points(points, start, max_ndx + 1, epsilon, new_points);
        simplify_points(points, max_ndx, end, epsilon, new_points);
    } else {
        if new_points.is_empty() {
            new_points.push(s);
        }
        new_points.push(e);
    }

    new_points.to_vec()
}

pub fn simplify<F>(points: &[Point2D<F>], distance: F) -> Vec<Point2D<F>>
where
    F: Float + Display,
{
    simplify_points(points, 0, points.len(), distance, &mut vec![])
}

pub fn get_points_on_bezier_curve_with_splitting<F>(
    points: &[Point2D<F>],
    offset: usize,
    tolerance: F,
    new_points: &mut Vec<Point2D<F>>,
) -> Vec<Point2D<F>>
where
    F: Float + MulAssign,
{
    if flatness(points, offset) < tolerance {
        let p0 = points[offset + 0];
        if !new_points.is_empty() {
            let d = new_points.last().unwrap().distance_to(p0);
            if d > F::one() {
                new_points.push(p0);
            }
        } else {
            new_points.push(p0);
        }
        new_points.push(points[offset + 3]);
    } else {
        let t = F::from(0.5).unwrap();
        let p1 = points[offset + 0];
        let p2 = points[offset + 1];
        let p3 = points[offset + 2];
        let p4 = points[offset + 3];

        let q1 = p1.lerp(p2, t);
        let q2 = p2.lerp(p3, t);
        let q3 = p3.lerp(p4, t);

        let r1 = q1.lerp(q2, t);
        let r2 = q2.lerp(q3, t);

        let red = r1.lerp(r2, t);

        get_points_on_bezier_curve_with_splitting(&[p1, q1, r1, red], 0, tolerance, new_points);
        get_points_on_bezier_curve_with_splitting(&[red, r2, q3, p4], 0, tolerance, new_points);
    }

    return new_points.to_vec();
}

pub fn points_on_bezier_curves<F>(
    points: &[Point2D<F>],
    tolerance: F,
    distance: F,
) -> Vec<Point2D<F>>
where
    F: Float + MulAssign + Display,
{
    let mut new_points = vec![];
    let num_segments = points.len() / 3;
    for i in 0..num_segments {
        let offset = i * 3;
        get_points_on_bezier_curve_with_splitting(points, offset, tolerance, &mut new_points);
    }

    if distance > F::zero() {
        return simplify_points(&new_points, 0, new_points.len(), distance, &mut vec![]);
    }
    return new_points;
}

pub fn curve_to_bezier<F>(points_in: &[Point2D<F>], curve_tightness: F) -> Option<Vec<Point2D<F>>>
where
    F: Float,
{
    if points_in.len() < 3 {
        None
    } else {
        let mut out = vec![];
        if points_in.len() == 3 {
            out.extend_from_slice(points_in);
            out.push(*points_in.last().unwrap());
        } else {
            let mut points = vec![];
            points.push(points_in[0]);
            points.push(points_in[0]);
            for i in 1..points_in.len() {
                points.push(points_in[1]);
                if i == points_in.len() - 1 {
                    points.push(points_in[i]);
                }
            }
            let s = F::one() - curve_tightness;
            out.push(points[0]);
            for i in 1..points.len() - 2 {
                let cached_point = points[i];
                //let b_0  = cached_point.clone();
                let b_1 = point2(
                    cached_point.x
                        + (s * points[i + 1].x - s * points[i - 1].x) / F::from(6).unwrap(),
                    cached_point.y
                        + (s * points[i + 1].y - s * points[i - 1].y) / F::from(6).unwrap(),
                );
                let b_2 = point2(
                    points[i + 1].x + (s * points[i].x - s * points[i + 2].x) / F::from(6).unwrap(),
                    points[i + 1].y + (s * points[i].y - s * points[i + 2].y) / F::from(6).unwrap(),
                );
                let b_3 = point2(points[i + 1].x, points[i + 1].y);
                out.push(b_1);
                out.push(b_2);
                out.push(b_3);
            }
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use euclid::point2;

    #[test]
    fn distance_to_segment_squared() {
        let expected = 1.0;
        let result = super::distance_to_segment_squared(
            point2(0.0, 1.0),
            point2(-1.0, 0.0),
            point2(1.0, 0.0),
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn flatness() {
        let expected = 9.0;
        let result = super::flatness(
            &[
                point2(0.0, 1.0),
                point2(1.0, 3.0),
                point2(2.0, 3.0),
                point2(3.0, 4.0),
            ],
            0,
        );
        assert_eq!(expected, result);
    }
}

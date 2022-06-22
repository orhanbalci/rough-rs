use std::cmp::{max, min, Ord};
use std::ops::MulAssign;

use euclid::default::Point2D;
use num_traits::Float;

/// computes distance squared from a point p to the line segment vw
pub fn distance_to_segment_squared<F>(p: &Point2D<F>, v: &Point2D<F>, w: &Point2D<F>) -> F
where
    F: Float + Ord,
{
    let l2 = v.distance_to(*w);
    if l2 == F::zero() {
        p.distance_to(*v).powi(2)
    } else {
        let mut t = ((p.x - v.x) * (w.x - v.x) + (p.y - v.y) * (w.y - v.y)) / l2;
        t = max(F::zero(), min(F::one(), t));
        p.distance_to(v.lerp(*w, t))
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
    F: Float + Ord,
{
    let s = points[start];
    let e = points[end - 1];
    let mut max_dist_sq = F::zero();
    let mut max_ndx = 0;
    for i in start + 1..end - 1 {
        let distance_sq = distance_to_segment_squared(&points[i], &s, &e);
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
    F: Float + Ord,
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
    F: Float + Ord + MulAssign,
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
    F: Float + Ord + MulAssign,
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

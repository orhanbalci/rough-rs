use euclid::{default::Point2D, Trig};
use num_traits::Float;
use std::cmp::Ord;

// computes distance squared from a point p to the line segment vw
pub fn distance_to_segment_squared<F>(p: &Point2D<F>, v: &Point2D<F>, w: &Point2D<F>) -> F
where
    F: Float + Trig + Ord,
{
    let l2 = v.distance_to(*w);
    if l2 == F::zero() {
        p.distance_to(*v).powi(2)
    } else {
        let mut t = ((p.x - v.x) * (w.x - v.x) + (p.y - v.y) * (w.y - v.y)) / l2;
        t = std::cmp::max(F::zero(), std::cmp::min(F::one(), t));
        p.distance_to(v.lerp(*w, t))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

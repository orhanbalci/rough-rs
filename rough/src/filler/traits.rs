use euclid::{default::Point2D, Trig};
use num_traits::{Float, FromPrimitive};

use crate::core::{OpSet, Options};

pub trait PatternFiller<F: Float + Trig + FromPrimitive> {
    fn fill_polygons(polygon_list: Vec<Vec<Point2D<F>>>, o: &mut Options) -> OpSet<F>;
}

use euclid::default::Point2D;
use euclid::Trig;
use num_traits::{Float, FromPrimitive};
use svgtypes::{PathParser, PathSegment};

pub fn points_on_path<F>(
    path: String,
    tolerance: Option<f32>,
    distance: Option<f32>,
) -> Vec<Vec<Point2D<F>>>
where
    F: FromPrimitive + Trig + Float,
{
    let path_parser = PathParser::from(path.as_ref());
    let path_segments: Vec<PathSegment> = path_parser.flatten().collect();
    return vec![];
}

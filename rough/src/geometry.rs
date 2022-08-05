use euclid::default::Point2D;
use euclid::{Angle, Translation2D, Trig, Vector2D};
use num_traits::Float;

#[derive(Clone, Debug, PartialEq)]
pub struct Line<F: Float + Trig> {
    pub start_point: Point2D<F>,
    pub end_point: Point2D<F>,
}

impl<F: Float + Trig> Line<F> {
    pub fn from(points: &[Point2D<F>]) -> Self {
        Line {
            start_point: points[0],
            end_point: points[1],
        }
    }
    pub fn as_points(&self) -> Vec<Point2D<F>> {
        return vec![self.start_point, self.end_point];
    }

    pub fn length(&self) -> F {
        (self.end_point - self.start_point).length()
    }

    pub fn rotate(&mut self, center: &Point2D<F>, degrees: F) {
        let angle = Angle::radians(degrees.to_radians());
        let translation = Translation2D::new(-center.x, -center.y);
        let transformation = translation
            .to_transform()
            .then_rotate(angle)
            .then_translate(Vector2D::new(center.x, center.y));
        self.start_point = transformation.transform_point(self.start_point);
        self.end_point = transformation.transform_point(self.end_point);
    }
}

pub fn rotate_points<F: Float + Trig>(
    points: &[Point2D<F>],
    center: &Point2D<F>,
    degrees: F,
) -> Vec<Point2D<F>> {
    let angle = Angle::radians(degrees.to_radians());
    let translation = Translation2D::new(-center.x, -center.y);
    let transformation = translation
        .to_transform()
        .then_rotate(angle)
        .then_translate(Vector2D::new(center.x, center.y));
    return points
        .iter()
        .map(|&p| transformation.transform_point(p))
        .collect::<Vec<Point2D<F>>>();
}

pub fn rotate_lines<F: Float + Trig>(
    lines: &[Line<F>],
    center: &Point2D<F>,
    degrees: F,
) -> Vec<Line<F>> {
    lines
        .iter()
        .cloned()
        .map(|mut l| {
            l.rotate(center, degrees);
            l
        })
        .collect::<Vec<Line<F>>>()
}

#[cfg(test)]
mod tests {
    use euclid::default::Point2D;
    #[test]
    fn line_length() {
        let l = super::Line::from(&[Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0)]);
        assert_eq!(l.length(), f32::sqrt(2.0));
    }
}

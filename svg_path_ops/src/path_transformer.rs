use std::collections::HashMap;

use cgmath::{Angle, Deg, Matrix3, Vector2, Vector3};
use svgtypes::{PathParser, PathSegment, TransformListParser, TransformListToken};

pub struct PathTransformer {
    path_segments: Vec<PathSegment>,
    stack: Vec<Matrix3<f64>>,
}

impl PathTransformer {
    pub fn new(path: String) -> Self {
        let mut path_parser = PathParser::from(path.as_ref());
        if path_parser.any(|a| a.is_err()) {
            panic!("unexpected path string. can not parse it.")
        }

        PathTransformer {
            path_segments: path_parser
                .filter(|ps| ps.is_ok())
                .map(|ps| ps.unwrap())
                .collect(),
            stack: Vec::new(),
        }
    }

    pub fn translate(&mut self, tx: f64, ty: f64) -> &mut Self {
        self.stack
            .push(Matrix3::from_translation(Vector2::new(tx, ty)));
        self
    }

    pub fn scale(&mut self, sx: f64, sy: f64) -> &mut Self {
        self.stack.push(Matrix3::from_nonuniform_scale(sx, sy));
        self
    }

    pub fn rotate(&mut self, angle: f64, rx: f64, ry: f64) -> &mut Self {
        self.stack.push(Matrix3::from_axis_angle(
            Vector3::new(rx, ry, 0.0),
            Deg(angle),
        ));
        self
    }

    pub fn skew_x(&mut self, degrees: f64) -> &mut Self {
        let skew_xmatrix = Matrix3::new(1.0, 0.0, 0.0, Deg(degrees).tan(), 1.0, 0.0, 0.0, 0.0, 1.0);
        self.stack.push(skew_xmatrix);
        self
    }

    pub fn skew_y(&mut self, degrees: f64) -> &mut Self {
        let skew_ymatrix = Matrix3::new(1.0, Deg(degrees).tan(), 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
        self.stack.push(skew_ymatrix);
        self
    }

    pub fn matrix(&mut self, matrix: [f64; 6]) -> &mut Self {
        let converted = Matrix3::new(
            matrix[0], matrix[1], 0.0, matrix[2], matrix[3], 0.0, matrix[4], matrix[5], 1.0,
        );
        self.stack.push(converted);
        self
    }

    pub fn transform(&mut self, transform: String) -> &mut Self {
        let parser = TransformListParser::from(transform.as_str());
        for path_transform in parser {
            match path_transform {
                Ok(pt) => {
                    self.apply_token(pt);
                }
                Err(_) => {
                    println!("Can not parse transform string.");
                }
            }
        }
        self
    }

    fn apply_token(&mut self, token: TransformListToken) -> &mut Self {
        match token {
            TransformListToken::Matrix { a, b, c, d, e, f } => self.matrix([a, b, c, d, e, f]),
            TransformListToken::Translate { tx, ty } => self.translate(tx, ty),
            TransformListToken::Scale { sx, sy } => self.scale(sx, sy),
            TransformListToken::Rotate { angle } => self.rotate(angle, 0.0, 0.0),
            TransformListToken::SkewX { angle } => self.skew_x(angle),
            TransformListToken::SkewY { angle } => self.skew_y(angle),
        };

        self
    }

    fn evaluate_stack(&mut self) -> &mut Self {
        if self.stack.len() == 0 {
            return self;
        } else {
            if self.stack.len() == 1 {
                let single_transformation = self.stack.pop().expect("empty transformation stack");
                self.apply_matrix(single_transformation);
                return self;
            } else {
                let mut combined = Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
                while !self.stack.is_empty() {
                    combined = combined
                        * self
                            .stack
                            .pop()
                            .expect("can not find transformation matrix");
                }
                self.apply_matrix(combined);
                return self;
            }
        }
    }

    fn apply_matrix(&mut self, final_matrix: Matrix3<f64>) -> &mut Self {
        self.iterate(|segment, pos, x, y| {
            let result: PathSegment;
            match *segment {
                PathSegment::MoveTo { abs, x: seg_x, y: seg_y } => {
                    if abs {
                        let p = final_matrix * Vector3::new(seg_x, seg_y, 1.0);
                        result = PathSegment::MoveTo { abs: true, x: p.x, y: p.y }
                    } else {
                        // Edge case. The very first `m` should be processed as absolute, if happens.
                        // Make sense for coord shift transforms.
                        let is_relative = pos > 0;
                        let p = final_matrix
                            * Vector3::new(seg_x, seg_y, if is_relative { 0.0 } else { 1.0 });

                        result = PathSegment::MoveTo { abs: false, x: p.x, y: p.y };
                    }
                }
                PathSegment::LineTo { abs, x: seg_x, y: seg_y } => {
                    let p = final_matrix * Vector3::new(seg_x, seg_y, if abs { 1.0 } else { 0.0 });
                    result = PathSegment::LineTo { abs, x: p.x, y: p.y }
                }
                PathSegment::HorizontalLineTo { abs, x: seg_x } => {
                    if abs {
                        let p = final_matrix * Vector3::new(seg_x, y, 1.0);
                        result = if p.x == (final_matrix * Vector3::new(x, y, 0.0)).y {
                            PathSegment::HorizontalLineTo { abs: true, x: p.x }
                        } else {
                            PathSegment::LineTo { abs: true, x: p.x, y: p.y }
                        }
                    } else {
                        let p = final_matrix * Vector3::new(seg_x, 0.0, 0.0);

                        result = if p.y == 0.0 {
                            PathSegment::HorizontalLineTo { abs: false, x: p.x }
                        } else {
                            PathSegment::LineTo { abs: false, x: p.x, y: p.y }
                        };
                    }
                }
                PathSegment::VerticalLineTo { abs, y: seg_y } => {
                    if abs {
                        let p = final_matrix * Vector3::new(x, seg_y, 1.0);
                        result = if p.x == (final_matrix * Vector3::new(x, y, 0.0)).x {
                            PathSegment::VerticalLineTo { abs: true, y: p.y }
                        } else {
                            PathSegment::LineTo { abs: true, x: p.x, y: p.y }
                        };
                    } else {
                        let p = final_matrix * Vector3::new(x, seg_y, 0.0);
                        result = if p.x == 0.0 {
                            PathSegment::VerticalLineTo { abs: false, y: p.y }
                        } else {
                            PathSegment::LineTo { abs: false, x: p.x, y: p.y }
                        };
                    }
                }
                PathSegment::CurveTo { abs, x1, y1, x2, y2, x: seg_x, y: seg_y } => {
                    let p1 = final_matrix * Vector3::new(x1, y1, if abs { 1.0 } else { 0.0 });
                    let p2 = final_matrix * Vector3::new(x2, y2, if abs { 1.0 } else { 0.0 });
                    let p3 = final_matrix * Vector3::new(seg_x, seg_y, if abs { 1.0 } else { 0.0 });
                    result = PathSegment::CurveTo {
                        abs,
                        x1: p1.x,
                        y1: p1.y,
                        x2: p2.x,
                        y2: p2.y,
                        x: p3.x,
                        y: p3.y,
                    };
                }
                PathSegment::SmoothCurveTo { abs, x2, y2, x: seg_x, y: seg_y } => {
                    let p2 = final_matrix * Vector3::new(x2, y2, if abs { 1.0 } else { 0.0 });
                    let p3 = final_matrix * Vector3::new(seg_x, seg_y, if abs { 1.0 } else { 0.0 });
                    result =
                        PathSegment::SmoothCurveTo { abs, x2: p2.x, y2: p2.y, x: p3.x, y: p3.y };
                }
                PathSegment::Quadratic { abs, x1, y1, x: seg_x, y: seg_y } => {
                    let p1 = final_matrix * Vector3::new(x1, y1, if abs { 1.0 } else { 0.0 });
                    let p2 = final_matrix * Vector3::new(seg_x, seg_y, if abs { 1.0 } else { 0.0 });
                    result = PathSegment::Quadratic { abs, x1: p1.x, y1: p1.y, x: p2.x, y: p2.y };
                }
                PathSegment::SmoothQuadratic { abs, x: seg_x, y: seg_y } => {
                    let p2 = final_matrix * Vector3::new(seg_x, seg_y, if abs { 1.0 } else { 0.0 });
                    result = PathSegment::SmoothQuadratic { abs, x: p2.x, y: p2.y }
                }

                PathSegment::EllipticalArc {
                    abs,
                    rx,
                    ry,
                    x_axis_rotation,
                    large_arc,
                    sweep,
                    x,
                    y,
                } => todo!(),
                PathSegment::ClosePath { abs } => result = PathSegment::ClosePath { abs },
            };
            vec![result]
        });
        return self;
    }

    fn iterate<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&PathSegment, usize, f64, f64) -> Vec<PathSegment>,
    {
        let mut last_x: f64 = 0.0;
        let mut last_y: f64 = 0.0;
        let mut contour_start_x: f64 = 0.0;
        let mut contour_start_y: f64 = 0.0;
        let mut replacements = HashMap::new();

        for (pos, segment) in self.path_segments.iter().enumerate() {
            let transformation_result = func(segment, pos, last_x, last_y);

            if !transformation_result.is_empty() {
                replacements.insert(pos, transformation_result);
            }

            match segment {
                PathSegment::MoveTo { abs, x, y } => {
                    last_x = x + if *abs { 0.0 } else { last_x };
                    last_y = y + if *abs { 0.0 } else { last_y };
                    contour_start_x = last_x;
                    contour_start_y = last_y;
                }
                PathSegment::LineTo { abs, x, y } => {
                    last_x = x + if *abs { 0.0 } else { last_x };
                    last_y = y + if *abs { 0.0 } else { last_y };
                }
                PathSegment::HorizontalLineTo { abs, x } => {
                    last_x = x + if *abs { 0.0 } else { last_x };
                }
                PathSegment::VerticalLineTo { abs, y } => {
                    last_y = y + if *abs { 0.0 } else { last_y };
                }
                PathSegment::CurveTo { abs, x1: _, y1: _, x2: _, y2: _, x, y } => {
                    last_x = x + if *abs { 0.0 } else { last_x };
                    last_y = y + if *abs { 0.0 } else { last_y };
                }
                PathSegment::SmoothCurveTo { abs, x2: _, y2: _, x, y } => {
                    last_x = x + if *abs { 0.0 } else { last_x };
                    last_y = y + if *abs { 0.0 } else { last_y };
                }
                PathSegment::Quadratic { abs, x1: _, y1: _, x, y } => {
                    last_x = x + if *abs { 0.0 } else { last_x };
                    last_y = y + if *abs { 0.0 } else { last_y };
                }
                PathSegment::SmoothQuadratic { abs, x, y } => {
                    last_x = x + if *abs { 0.0 } else { last_x };
                    last_y = y + if *abs { 0.0 } else { last_y };
                }
                PathSegment::EllipticalArc {
                    abs,
                    rx: _,
                    ry: _,
                    x_axis_rotation: _,
                    large_arc: _,
                    sweep: _,
                    x,
                    y,
                } => {
                    last_x = x + if *abs { 0.0 } else { last_x };
                    last_y = y + if *abs { 0.0 } else { last_y };
                }
                PathSegment::ClosePath { abs: _ } => {
                    last_x = contour_start_x;
                    last_y = contour_start_y;
                }
            }
        }

        if replacements.len() == 0 {
            return self;
        } else {
            let mut updated_segments = vec![];
            for i in (0..self.path_segments.len()) {
                if replacements.contains_key(&i) {
                    let replacing_segments =
                        replacements.get(&i).expect("can not retrieve replacement");
                    replacing_segments
                        .iter()
                        .for_each(|r| updated_segments.push(*r));
                } else {
                    updated_segments.push(
                        *self
                            .path_segments
                            .get(i)
                            .expect("can not retrieve path segment"),
                    );
                }
            }

            self.path_segments = updated_segments;

            return self;
        }
    }
}

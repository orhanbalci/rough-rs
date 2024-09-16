use cgmath::{Angle, Deg, Matrix3, Vector2, Vector3};
use svgtypes::{PathParser, PathSegment};

pub struct PathTransformer {
    path_segments: Vec<PathSegment>,
    stack: Vec<Matrix3<f32>>,
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

    pub fn translate(&mut self, tx: f32, ty: f32) -> &mut Self {
        self.stack
            .push(Matrix3::from_translation(Vector2::new(tx, ty)));
        self
    }

    pub fn scale(&mut self, sx: f32, sy: f32) -> &mut Self {
        self.stack.push(Matrix3::from_nonuniform_scale(sx, sy));
        self
    }

    pub fn rotate(&mut self, angle: f32, rx: f32, ry: f32) -> &mut Self {
        self.stack.push(Matrix3::from_axis_angle(
            Vector3::new(rx, ry, 0.0),
            Deg(angle),
        ));
        self
    }

    pub fn skew_x(&mut self, degrees: f32) -> &mut Self {
        let skew_xmatrix = Matrix3::new(1.0, 0.0, 0.0, Deg(degrees).tan(), 1.0, 0.0, 0.0, 0.0, 1.0);
        self.stack.push(skew_xmatrix);
        self
    }

    pub fn skew_y(&mut self, degrees: f32) -> &mut Self {
        let skew_ymatrix = Matrix3::new(1.0, Deg(degrees).tan(), 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
        self.stack.push(skew_ymatrix);
        self
    }

    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        let converted = Matrix3::new(
            matrix[0], matrix[3], 0.0, matrix[1], matrix[4], 0.0, matrix[2], matrix[5], 1.0,
        );
        self.stack.push(converted);
        self
    }
}

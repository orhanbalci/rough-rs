use euclid::default::Point2D;
use euclid::Trig;
use num_traits::{Float, FromPrimitive};
use palette::Srgb;
use rand::rngs::StdRng;
use rand::{random, Rng, SeedableRng};

pub struct Space;

pub struct Config {
    options: Option<Options>,
}

pub struct DrawingSurface {
    width: f32,
    height: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FillStyle {
    Solid,
    Hachure,
    ZigZag,
    CrossHatch,
    Dots,
    Dashed,
    ZigZagLine,
}

#[derive(Clone, Builder)]
#[builder(setter(strip_option))]
pub struct Options {
    #[builder(default = "Some(2.0)")]
    pub max_randomness_offset: Option<f32>,
    #[builder(default = "Some(1.0)")]
    pub roughness: Option<f32>,
    #[builder(default = "Some(2.0)")]
    pub bowing: Option<f32>,
    #[builder(default = "Some(Srgb::new(0.0, 0.0, 0.0))")]
    pub stroke: Option<Srgb>,
    #[builder(default = "Some(1.0)")]
    pub stroke_width: Option<f32>,
    #[builder(default = "Some(0.95)")]
    pub curve_fitting: Option<f32>,
    #[builder(default = "Some(0.0)")]
    pub curve_tightness: Option<f32>,
    #[builder(default = "Some(9.0)")]
    pub curve_step_count: Option<f32>,
    #[builder(default = "None")]
    pub fill: Option<Srgb>,
    #[builder(default = "None")]
    pub fill_style: Option<FillStyle>,
    #[builder(default = "Some(-1.0)")]
    pub fill_weight: Option<f32>,
    #[builder(default = "Some(-41.0)")]
    pub hachure_angle: Option<f32>,
    #[builder(default = "Some(-1.0)")]
    pub hachure_gap: Option<f32>,
    #[builder(default = "Some(1.0)")]
    pub simplification: Option<f32>,
    #[builder(default = "Some(-1.0)")]
    pub dash_offset: Option<f32>,
    #[builder(default = "Some(-1.0)")]
    pub dash_gap: Option<f32>,
    #[builder(default = "Some(-1.0)")]
    pub zigzag_offset: Option<f32>,
    #[builder(default = "Some(345_u64)")]
    pub seed: Option<u64>,
    #[builder(default = "None")]
    pub stroke_line_dash: Option<Vec<f64>>,
    #[builder(default = "None")]
    pub stroke_line_dash_offset: Option<f64>,
    #[builder(default = "None")]
    pub fill_line_dash: Option<Vec<f64>>,
    #[builder(default = "None")]
    pub fill_line_dash_offset: Option<f64>,
    #[builder(default = "Some(false)")]
    pub disable_multi_stroke: Option<bool>,
    #[builder(default = "Some(false)")]
    pub disable_multi_stroke_fill: Option<bool>,
    #[builder(default = "Some(false)")]
    pub preserve_vertices: Option<bool>,
    #[builder(default = "None")]
    pub fixed_decimal_place_digits: Option<f32>,
    #[builder(default = "None")]
    pub randomizer: Option<StdRng>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            max_randomness_offset: Some(2.0),
            roughness: Some(1.0),
            bowing: Some(2.0),
            stroke: Some(Srgb::new(0.0, 0.0, 0.0)),
            stroke_width: Some(1.0),
            curve_tightness: Some(0.0),
            curve_fitting: Some(0.95),
            curve_step_count: Some(9.0),
            fill: None,
            fill_style: None,
            fill_weight: Some(-1.0),
            hachure_angle: Some(-41.0),
            hachure_gap: Some(-1.0),
            dash_offset: Some(-1.0),
            dash_gap: Some(-1.0),
            zigzag_offset: Some(-1.0),
            seed: Some(345_u64),
            disable_multi_stroke: Some(false),
            disable_multi_stroke_fill: Some(false),
            preserve_vertices: Some(false),
            simplification: Some(1.0),
            stroke_line_dash: None,
            stroke_line_dash_offset: None,
            fill_line_dash: None,
            fill_line_dash_offset: None,
            fixed_decimal_place_digits: None,
            randomizer: None,
        }
    }
}

impl Options {
    pub fn random(&mut self) -> f64 {
        match &mut self.randomizer {
            Some(r) => r.gen(),
            None => match self.seed {
                Some(s) => {
                    let rnd = self.randomizer.insert(StdRng::seed_from_u64(s));
                    rnd.gen()
                }
                None => {
                    let rnd = self.randomizer.insert(StdRng::seed_from_u64(random()));
                    rnd.gen()
                }
            },
        }
    }

    pub fn set_hachure_angle(&mut self, angle: Option<f32>) -> &mut Self {
        self.hachure_angle = angle;
        self
    }

    pub fn set_hachure_gap(&mut self, gap: Option<f32>) -> &mut Self {
        self.hachure_gap = gap;
        self
    }
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub enum OpType {
    Move,
    BCurveTo,
    LineTo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpSetType {
    Path,
    FillPath,
    FillSketch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Op<F: Float + Trig> {
    pub op: OpType,
    pub data: Vec<F>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpSet<F: Float + Trig> {
    pub op_set_type: OpSetType,
    pub ops: Vec<Op<F>>,
    pub size: Option<Point2D<F>>,
    pub path: Option<String>,
}

pub struct Drawable<F: Float + Trig> {
    pub shape: String,
    pub options: Options,
    pub sets: Vec<OpSet<F>>,
}

pub struct PathInfo {
    pub d: String,
    pub stroke: Option<Srgb>,
    pub stroke_width: Option<f32>,
    pub fill: Option<Srgb>,
}

pub fn _c<U: Float + FromPrimitive>(inp: f32) -> U {
    U::from(inp).expect("can not parse from f32")
}

pub fn _cc<U: Float + FromPrimitive>(inp: f64) -> U {
    U::from(inp).expect("can not parse from f64")
}

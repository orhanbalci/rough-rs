use euclid::Trig;
use num_traits::{Float, FromPrimitive};
use piet::kurbo::{BezPath, PathEl, Point};
use rough::core::{Drawable, OpSet, OpSetType, OpType};
use rough::generator::Generator;

#[derive(Default)]
pub struct KurboGenerator {
    gen: Generator,
}

impl KurboGenerator {
    fn drawing_to_shape<F: Trig + Float + FromPrimitive>(drawable: &Drawable<F>) -> Vec<BezPath> {
        let mut result = vec![];
        for set in drawable.sets.iter() {
            match set.op_set_type {
                OpSetType::Path => {
                    result.push(KurboGenerator::opset_to_shape(&set));
                }
                OpSetType::FillPath => {
                    todo!("fill path not implemented");
                }
                OpSetType::FillSketch => {
                    //todo!("fill sketch not implemented");
                    result.push(KurboGenerator::opset_to_shape(&set));
                }
            }
        }
        return result;
    }

    fn opset_to_shape<F: Trig + Float + FromPrimitive>(op_set: &OpSet<F>) -> BezPath {
        let mut path: BezPath = BezPath::new();
        for item in op_set.ops.iter() {
            match item.op {
                OpType::Move => path.extend([PathEl::MoveTo(Point::new(
                    item.data[0].to_f64().unwrap(),
                    item.data[1].to_f64().unwrap(),
                ))]),
                OpType::BCurveTo => path.extend([PathEl::CurveTo(
                    Point::new(
                        item.data[0].to_f64().unwrap(),
                        item.data[1].to_f64().unwrap(),
                    ),
                    Point::new(
                        item.data[2].to_f64().unwrap(),
                        item.data[3].to_f64().unwrap(),
                    ),
                    Point::new(
                        item.data[4].to_f64().unwrap(),
                        item.data[5].to_f64().unwrap(),
                    ),
                )]),
                OpType::LineTo => {
                    path.extend([PathEl::LineTo(Point::new(
                        item.data[2].to_f64().unwrap(),
                        item.data[3].to_f64().unwrap(),
                    ))]);
                }
            }
        }
        return path;
    }

    pub fn line<F: Trig + Float + FromPrimitive>(
        &self,
        x1: F,
        y1: F,
        x2: F,
        y2: F,
    ) -> Vec<BezPath> {
        let drawable = self.gen.line(x1, y1, x2, y2);
        KurboGenerator::drawing_to_shape(&drawable)
    }

    pub fn rectangle<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
    ) -> Vec<BezPath> {
        let drawable = self.gen.rectangle(x, y, width, height);
        KurboGenerator::drawing_to_shape(&drawable)
    }

    pub fn ellipse<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
    ) -> Vec<BezPath> {
        let drawable = self.gen.ellipse(x, y, width, height);
        KurboGenerator::drawing_to_shape(&drawable)
    }
}

use std::borrow::BorrowMut;

use euclid::default::Point2D;
use euclid::Trig;
use num_traits::{Float, FromPrimitive};

use self::dashed_filler::DashedFiller;
use self::scan_line_hachure::ScanlineHachureFiller;
use self::traits::PatternFiller;

pub mod dashed_filler;
pub mod scan_line_hachure;
pub mod traits;

pub enum FillerType {
    ScanLineHachure,
    DashedFiller,
}

pub fn get_filler<'a, F, P>(f: FillerType) -> Box<dyn PatternFiller<F, P> + 'a>
where
    F: Float + Trig + FromPrimitive + 'a,
    P: BorrowMut<Vec<Vec<Point2D<F>>>>,
{
    match f {
        FillerType::ScanLineHachure => Box::new(ScanlineHachureFiller::new()),
        FillerType::DashedFiller => Box::new(DashedFiller::new()),
    }
}

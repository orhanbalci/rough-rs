use std::borrow::BorrowMut;

use euclid::default::Point2D;
use euclid::Trig;
use num_traits::{Float, FromPrimitive};

use self::scan_line_hachure::ScanlineHachureFiller;
use self::traits::PatternFiller;

pub mod scan_line_hachure;
pub mod traits;

pub enum FillerType {
    ScanLineHachure,
}

pub fn get_filler<F, P>(f: FillerType) -> impl PatternFiller<F, P>
where
    F: Float + Trig + FromPrimitive,
    P: BorrowMut<Vec<Vec<Point2D<F>>>>,
{
    match f {
        FillerType::ScanLineHachure => ScanlineHachureFiller::new(),
    }
}

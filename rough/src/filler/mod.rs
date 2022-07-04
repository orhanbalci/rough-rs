use euclid::Trig;
use num_traits::{Float, FromPrimitive};

use self::scan_line_hachure::ScanlineHachureFiller;
use self::traits::PatternFiller;

pub mod scan_line_hachure;
pub mod traits;

pub enum FillerType {
    ScanLineHachure,
}

pub fn get_filler<F>(f: FillerType) -> impl PatternFiller<F>
where
    F: Float + Trig + FromPrimitive,
{
    match f {
        FillerType::ScanLineHachure => ScanlineHachureFiller::new(),
    }
}

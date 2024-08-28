use ttf_parser::{self as ttf};

pub trait XmlWriterExt {
    fn write_color_attribute(&mut self, name: &str, ts: ttf::RgbaColor);
    fn write_transform_attribute(&mut self, name: &str, ts: ttf::Transform);
    fn write_spread_method_attribute(&mut self, method: ttf::colr::GradientExtend);
}

impl XmlWriterExt for xmlwriter::XmlWriter {
    fn write_color_attribute(&mut self, name: &str, color: ttf::RgbaColor) {}

    fn write_transform_attribute(&mut self, name: &str, ts: ttf::Transform) {}

    fn write_spread_method_attribute(&mut self, extend: ttf::colr::GradientExtend) {}
}

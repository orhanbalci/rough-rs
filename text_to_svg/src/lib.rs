#![feature(iter_map_windows)]

use std::path::Path;

use options::Options;
use outlinebuilder::Builder;
use ttf_parser::{self as ttf, Face};

mod options;
mod outlinebuilder;
mod xmlwriterext;

pub struct TextToSvg<'a> {
    pub font: Face<'a>,
}

impl<'a> TextToSvg<'a> {
    pub fn new<P>(font_data: &'a Vec<u8>) -> Self
    where
        P: AsRef<Path>,
    {
        let face: Face<'a> = ttf::Face::parse(&font_data, 0).expect("unable to parse font file");
        TextToSvg { font: face }
    }

    pub fn get_width(&self, text: String, options: Options) -> f32 {
        let font_size = options.fontsize;
        let kerning = options.kerning;
        let font_scale: f32 = font_size as f32 / self.font.units_per_em() as f32;

        let mut width: f32 = 0.0;

        width += text
            .chars()
            .map(|c| {
                let index = self.font.glyph_index(c);
                return index
                    .map(|i| self.font.glyph_hor_advance(i).unwrap_or_default())
                    .unwrap_or_default() as f32
                    * font_scale;
            })
            .sum::<f32>();

        if kerning {
            if let Some(kerning_table) = self.font.tables().kern {
                width += text
                    .chars()
                    .map_windows(|[char1, char2]| {
                        let index1 = self.font.glyph_index(*char1);
                        let index2 = self.font.glyph_index(*char2);
                        if index1.and(index2).is_some() {
                            if let Some(kern_subtable) = kerning_table.subtables.into_iter().next()
                            {
                                return kern_subtable
                                    .glyphs_kerning(index1.unwrap(), index2.unwrap())
                                    .unwrap_or_default()
                                    as f32;
                            }
                        }
                        return 0.0;
                    })
                    .sum::<f32>();
            }
        }

        if let Some(ls) = options.letter_spacing {
            width += text.chars().count() as f32 * (ls * font_size) as f32;
        } else if let Some(tr) = options.tracking {
            width += text.chars().count() as f32 * (tr as f32 / 1000.0) * font_size as f32;
        }

        return width;
    }

    pub fn get_height(&self, font_size: u16) -> f32 {
        let font_scale = font_size as f32 / self.font.units_per_em() as f32;
        return (self.font.ascender() - self.font.descender()) as f32 * font_scale;
    }

    fn glyph_to_path(
        &self,
        x: f64,
        y: f64,
        glyph_id: ttf::GlyphId,
        cell_size: f64,
        scale: f64,
        svg: &mut xmlwriter::XmlWriter,
        path_buf: &mut String,
    ) {
        path_buf.clear();
        let mut builder = Builder(path_buf);
        let bbox = match self.font.outline_glyph(glyph_id, &mut builder) {
            Some(v) => v,
            None => return,
        };
        builder.finish();

        let bbox_w = (bbox.x_max as f64 - bbox.x_min as f64) * scale;
        let dx = (cell_size - bbox_w) / 2.0;
        let y = y + cell_size + self.font.descender() as f64 * scale;

        let transform = format!("matrix({} 0 0 {} {} {})", scale, -scale, x + dx, y);

        svg.start_element("path");
        svg.write_attribute("d", path_buf);
        svg.write_attribute("transform", &transform);
        svg.end_element();

        {
            let bbox_h = (bbox.y_max as f64 - bbox.y_min as f64) * scale;
            let bbox_x = x + dx + bbox.x_min as f64 * scale;
            let bbox_y = y - bbox.y_max as f64 * scale;

            svg.start_element("rect");
            svg.write_attribute("x", &bbox_x);
            svg.write_attribute("y", &bbox_y);
            svg.write_attribute("width", &bbox_w);
            svg.write_attribute("height", &bbox_h);
            svg.write_attribute("fill", "none");
            svg.write_attribute("stroke", "green");
            svg.end_element();
        }
    }
}

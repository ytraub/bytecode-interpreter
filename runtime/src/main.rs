/* use ev3dev_lang_rust::motors::{LargeMotor, MotorPort};
use ev3dev_lang_rust::sensors::{ColorSensor, SensorPort}; */
use ev3dev_lang_rust::{Ev3Error, Ev3Result};
use glyph_brush_layout::{ab_glyph::*, *};

mod utils;

fn main() -> Ev3Result<()> {
    let dejavu = FontRef::try_from_slice(include_bytes!("../fonts/DejaVuSans.ttf")).unwrap_or_else(
        Ev3Error::InternalError {
            msg: "Invalid Font".to_string(),
        },
    );

    // Simple font mapping: FontId(0) -> deja vu sans, FontId(1) -> garamond
    let fonts = &[dejavu];
    let glyphs = Layout::default().calculate_glyphs(
        fonts,
        &SectionGeometry {
            screen_position: (150.0, 50.0),
            ..SectionGeometry::default()
        },
        &[
            SectionText {
                text: "hello ",
                scale: PxScale::from(20.0),
                font_id: FontId(0),
            },
            SectionText {
                text: "glyph_brush_layout",
                scale: PxScale::from(25.0),
                font_id: FontId(1),
            },
        ],
    );
    Ok(())
}

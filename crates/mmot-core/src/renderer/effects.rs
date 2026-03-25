use skia_safe::{color_filters, image_filters, Color, ColorFilter, ImageFilter};

use crate::schema::effects::Effect;

/// Build a chained Skia `ImageFilter` from a list of effects.
/// Each effect chains onto the previous one via the `input` parameter.
pub fn build_image_filter(effects: &[Effect]) -> Option<ImageFilter> {
    let mut filter: Option<ImageFilter> = None;

    for effect in effects {
        // Take ownership of the current filter so it can be passed as input
        // to the next effect in the chain.
        let prev = filter.take();

        let next: Option<ImageFilter> = match effect {
            Effect::GaussianBlur { radius } => {
                let sigma = *radius as f32;
                image_filters::blur(
                    (sigma, sigma),
                    skia_safe::TileMode::Clamp,
                    prev,
                    image_filters::CropRect::NO_CROP_RECT,
                )
            }
            Effect::DropShadow {
                color,
                offset_x,
                offset_y,
                blur,
                opacity,
            } => {
                let c = parse_color_with_opacity(color, *opacity);
                image_filters::drop_shadow(
                    (*offset_x as f32, *offset_y as f32),
                    (*blur as f32, *blur as f32),
                    c,
                    None::<skia_safe::ColorSpace>,
                    prev,
                    image_filters::CropRect::NO_CROP_RECT,
                )
            }
            Effect::Glow { radius, .. } => {
                // Glow: approximate as a blur (full glow compositing would need
                // a separate pass with additive blend, but a blur captures the
                // visual softening effect).
                let sigma = *radius as f32;
                image_filters::blur(
                    (sigma, sigma),
                    skia_safe::TileMode::Clamp,
                    prev,
                    image_filters::CropRect::NO_CROP_RECT,
                )
            }
            Effect::BrightnessContrast {
                brightness,
                contrast,
            } => {
                let b = (*brightness as f32) / 100.0;
                let c = (*contrast as f32 / 100.0) + 1.0;
                // 4x5 colour matrix (row-major, 20 floats)
                let matrix: [f32; 20] = [
                    c, 0.0, 0.0, 0.0, b,
                    0.0, c, 0.0, 0.0, b,
                    0.0, 0.0, c, 0.0, b,
                    0.0, 0.0, 0.0, 1.0, 0.0,
                ];
                let cf = color_filters::matrix_row_major(&matrix, None);
                image_filters::color_filter(
                    cf,
                    prev,
                    image_filters::CropRect::NO_CROP_RECT,
                )
            }
            Effect::HueSaturation {
                saturation,
                lightness,
                ..
            } => {
                let s = 1.0 + (*saturation as f32 / 100.0);
                let l = *lightness as f32 / 100.0;
                // Luminance-preserving saturation matrix
                let matrix: [f32; 20] = [
                    0.213 + 0.787 * s, 0.715 - 0.715 * s, 0.072 - 0.072 * s, 0.0, l,
                    0.213 - 0.213 * s, 0.715 + 0.285 * s, 0.072 - 0.072 * s, 0.0, l,
                    0.213 - 0.213 * s, 0.715 - 0.715 * s, 0.072 + 0.928 * s, 0.0, l,
                    0.0,               0.0,               0.0,               1.0, 0.0,
                ];
                let cf = color_filters::matrix_row_major(&matrix, None);
                image_filters::color_filter(
                    cf,
                    prev,
                    image_filters::CropRect::NO_CROP_RECT,
                )
            }
            Effect::Invert => {
                let matrix: [f32; 20] = [
                    -1.0, 0.0,  0.0,  0.0, 1.0,
                    0.0,  -1.0, 0.0,  0.0, 1.0,
                    0.0,  0.0,  -1.0, 0.0, 1.0,
                    0.0,  0.0,  0.0,  1.0, 0.0,
                ];
                let cf = color_filters::matrix_row_major(&matrix, None);
                image_filters::color_filter(
                    cf,
                    prev,
                    image_filters::CropRect::NO_CROP_RECT,
                )
            }
            Effect::Tint { color, amount } => {
                let c = parse_color_with_opacity(color, *amount);
                let cf: Option<ColorFilter> =
                    color_filters::blend(c, skia_safe::BlendMode::Color);
                cf.and_then(|cf| {
                    image_filters::color_filter(
                        cf,
                        prev,
                        image_filters::CropRect::NO_CROP_RECT,
                    )
                })
            }
            Effect::Fill { color, opacity } => {
                let c = parse_color_with_opacity(color, *opacity);
                let cf: Option<ColorFilter> =
                    color_filters::blend(c, skia_safe::BlendMode::SrcIn);
                cf.and_then(|cf| {
                    image_filters::color_filter(
                        cf,
                        prev,
                        image_filters::CropRect::NO_CROP_RECT,
                    )
                })
            }
        };

        if next.is_some() {
            filter = next;
        }
    }

    filter
}

fn parse_color_with_opacity(hex: &str, opacity: f64) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(hex.get(0..2).unwrap_or("ff"), 16).unwrap_or(255);
    let g = u8::from_str_radix(hex.get(2..4).unwrap_or("ff"), 16).unwrap_or(255);
    let b = u8::from_str_radix(hex.get(4..6).unwrap_or("ff"), 16).unwrap_or(255);
    Color::from_argb((opacity.clamp(0.0, 1.0) * 255.0) as u8, r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_color_full_opacity() {
        let c = parse_color_with_opacity("#ff0000", 1.0);
        assert_eq!(c.r(), 255);
        assert_eq!(c.g(), 0);
        assert_eq!(c.b(), 0);
        assert_eq!(c.a(), 255);
    }

    #[test]
    fn parse_color_half_opacity() {
        let c = parse_color_with_opacity("#00ff00", 0.5);
        assert_eq!(c.r(), 0);
        assert_eq!(c.g(), 255);
        assert_eq!(c.b(), 0);
        assert_eq!(c.a(), 127); // 0.5 * 255 = 127
    }

    #[test]
    fn empty_effects_returns_none() {
        let effects: Vec<Effect> = vec![];
        assert!(build_image_filter(&effects).is_none());
    }

    #[test]
    fn blur_builds_filter() {
        let effects = vec![Effect::GaussianBlur { radius: 5.0 }];
        let filter = build_image_filter(&effects);
        assert!(filter.is_some(), "GaussianBlur should produce a filter");
    }

    #[test]
    fn invert_builds_filter() {
        let effects = vec![Effect::Invert];
        let filter = build_image_filter(&effects);
        assert!(filter.is_some(), "Invert should produce a filter");
    }

    #[test]
    fn chained_effects_build_filter() {
        let effects = vec![
            Effect::GaussianBlur { radius: 3.0 },
            Effect::Invert,
        ];
        let filter = build_image_filter(&effects);
        assert!(filter.is_some(), "chained effects should produce a filter");
    }
}

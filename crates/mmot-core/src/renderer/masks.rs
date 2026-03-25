use skia_safe::{Canvas, Path, Rect, RRect};

use crate::schema::effects::{Mask, MaskMode, MaskPath};

/// Apply masks to the canvas as clip regions.
pub fn apply_masks(canvas: &Canvas, masks: &[Mask]) {
    for mask in masks {
        let path = mask_path_to_skia(&mask.path);

        let clip_op = match mask.mode {
            MaskMode::Add => skia_safe::ClipOp::Intersect,
            MaskMode::Subtract => skia_safe::ClipOp::Difference,
            MaskMode::Intersect => skia_safe::ClipOp::Intersect,
            MaskMode::Difference => skia_safe::ClipOp::Difference,
        };

        // Anti-alias when feathering is requested
        canvas.clip_path(&path, clip_op, mask.feather > 0.0);
    }
}

fn mask_path_to_skia(mask_path: &MaskPath) -> Path {
    let mut path = Path::new();
    match mask_path {
        MaskPath::Rect {
            x,
            y,
            width,
            height,
            corner_radius,
        } => {
            let rect = Rect::from_xywh(*x as f32, *y as f32, *width as f32, *height as f32);
            if *corner_radius > 0.0 {
                let rrect =
                    RRect::new_rect_xy(rect, *corner_radius as f32, *corner_radius as f32);
                path.add_rrect(rrect, None);
            } else {
                path.add_rect(rect, None);
            }
        }
        MaskPath::Ellipse { cx, cy, rx, ry } => {
            let oval_rect = Rect::from_xywh(
                (*cx - *rx) as f32,
                (*cy - *ry) as f32,
                (*rx * 2.0) as f32,
                (*ry * 2.0) as f32,
            );
            path.add_oval(oval_rect, None);
        }
        MaskPath::Path { points, closed } => {
            if let Some(first) = points.first() {
                path.move_to((first[0] as f32, first[1] as f32));
                for pt in &points[1..] {
                    path.line_to((pt[0] as f32, pt[1] as f32));
                }
                if *closed {
                    path.close();
                }
            }
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_mask_to_skia_path() {
        let mask_path = MaskPath::Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            corner_radius: 0.0,
        };
        let path = mask_path_to_skia(&mask_path);
        assert!(!path.is_empty());
    }

    #[test]
    fn ellipse_mask_to_skia_path() {
        let mask_path = MaskPath::Ellipse {
            cx: 50.0,
            cy: 50.0,
            rx: 30.0,
            ry: 20.0,
        };
        let path = mask_path_to_skia(&mask_path);
        assert!(!path.is_empty());
    }

    #[test]
    fn polygon_mask_to_skia_path() {
        let mask_path = MaskPath::Path {
            points: vec![[0.0, 0.0], [100.0, 0.0], [50.0, 100.0]],
            closed: true,
        };
        let path = mask_path_to_skia(&mask_path);
        assert!(!path.is_empty());
    }
}

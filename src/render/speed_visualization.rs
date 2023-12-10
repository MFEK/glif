use MFEKmath::{Piecewise, Bezier, ArcLengthParameterization, Parameterization, Evaluate, Vector, mfek::ResolveCubic};
use glifrenderer::toggles::PreviewMode;
use skia_safe::{Canvas, Color, Paint};

use crate::{editor::Editor, user_interface::Interface, get_contour};

pub fn draw_velocity(v: &Editor, i: &Interface, canvas: &mut Canvas) {
    if i.viewport.preview_mode == PreviewMode::Paper { return };
    if i.curvature_vis == false { return };
    
    if let Some((cidx, _)) = v.selected_point() {
        let selected_contour = &get_contour!(v.get_active_layer_ref(), cidx);

        let scaling_factor: f64 = 200.;
        let quality = 6.;
        
        let piecewise: Piecewise<Bezier> = selected_contour.to_cubic().into();
        let mut first = false;
        for bez in piecewise.segs {
            let arclen_param = ArcLengthParameterization::from(&bez, 1000);
            let total_len = arclen_param.get_total_arclen();

            let samples = (quality * total_len as f32 * i.viewport.factor) as u32;
            let max_velocity_magnitude = 0.2;
            let min_velocity_magnitude = 0.0;

            // Function to map velocity magnitude to a color
            let alpha = 32u8; // Semi-transparent
            let map_velocity_to_color = |velocity_magnitude: f64| -> Color {
                let normalized = (velocity_magnitude.abs() - min_velocity_magnitude) / (max_velocity_magnitude - min_velocity_magnitude);
                let red = (normalized * 255.0) as u8;
                let blue = ((1.0 - normalized) * 255.0) as u8;
                Color::from_argb(alpha, red, 0, blue) // Gradient from blue to red
            };
    
            let derivative = bezier_derivative(&bez);
            // Second pass to draw the lines with gradient colors
            let start = if first { 0 } else { 1 };
            first = false;
            for sample in start..=samples {
                let t = sample as f64 / samples as f64;
                let t = arclen_param.parameterize(t);
            
                // Calculate the first and second derivatives
                let first_derivative = eval_quadratic_bezier(derivative.0, derivative.1, derivative.2, t);
                let second_derivative = eval_linear_bezier(bezier_second_derivative(&bez).0, bezier_second_derivative(&bez).1, t);
            
                // Calculate the curvature
                let curvature = (first_derivative.x * second_derivative.y - first_derivative.y * second_derivative.x) /
                                (first_derivative.x.powi(2) + first_derivative.y.powi(2)).powf(1.5);
            
                let scaled_curvature = curvature.signum() * curvature.abs().sqrt();
                let clamped_curvature = scaled_curvature.signum() * scaled_curvature.abs().min(max_velocity_magnitude);
                let normal = Vector { x: -first_derivative.y, y: first_derivative.x }.normalize();
            
                // Scale the normal by the curvature
                let scaled_normal = -normal * scaled_curvature * scaling_factor;
            
                let color = map_velocity_to_color(clamped_curvature);

                // Draw the normal line
                let point_on_curve = bez.at(t);
                let start = point_on_curve;
                let end = point_on_curve + scaled_normal;
                let mut normal_paint = Paint::default();
                normal_paint.set_color(color);
                normal_paint.set_stroke_width(1.0 / i.viewport.factor);
                normal_paint.set_anti_alias(true);
                canvas.draw_line(start.to_skia_point(), end.to_skia_point(), &normal_paint);
            }
            
        }
    }
}


// Function to calculate the derivative of a cubic Bézier curve
fn bezier_derivative(bez: &Bezier) -> (Vector, Vector, Vector) {
    // For a cubic Bézier curve, the derivative is a quadratic curve with these control points
    let dp0 = (bez.w2 - bez.w1) * 3.0;
    let dp1 = (bez.w3 - bez.w2) * 3.0;
    let dp2 = (bez.w4 - bez.w3) * 3.0;
    (dp0, dp1, dp2)
}

// Function to evaluate a quadratic Bézier curve at t
fn eval_quadratic_bezier(p0: Vector, p1: Vector, p2: Vector, t: f64) -> Vector {
    let one_minus_t = 1.0 - t as f64;
    p0 * one_minus_t.powi(2) + p1 * 2.0 * one_minus_t * t + p2 * t.powi(2)
}

// Function to calculate the second derivative of a cubic Bézier curve
fn bezier_second_derivative(bez: &Bezier) -> (Vector, Vector) {
    // The second derivative of a cubic Bézier curve is a linear curve with these control points
    let ddp0 = (bez.w3 - bez.w2 * 2.0 + bez.w1) * 6.0;
    let ddp1 = (bez.w4 - bez.w3 * 2.0 + bez.w2) * 6.0;
    (ddp0, ddp1)
}

// Function to evaluate a linear Bézier curve at t (which is essentially linear interpolation)
fn eval_linear_bezier(p0: Vector, p1: Vector, t: f64) -> Vector {
    p0 * (1.0 - t as f64) + p1 * t as f64
}
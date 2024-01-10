use glifrenderer::constants::OUTLINE_STROKE_THICKNESS;
use skia_safe::{Canvas, Font, Matrix, Paint, Path, Point, Rect};
use MFEKmath::{vec2, Vector};

use crate::editor::Editor;
use crate::tools::cut::{Cut, Intersection};
use crate::user_interface::Interface;
use glifrenderer::constants::MEASURE_STROKE;

pub struct Measure {
    pub start_point: Option<(f32, f32)>,
    pub end_point: Option<(f32, f32)>,
    pub enabled: bool,
}

impl Measure {
    pub fn draw_line(&self, i: &Interface, v: &Editor, canvas: &Canvas, factor: f32) {
        if !self.enabled {
            return;
        }
        if self.start_point.is_none() || self.end_point.is_none() {
            return;
        }

        let start_point: Vector = self.start_point.unwrap().into();
        let end_point: Vector = self.end_point.unwrap().into();

        // Calculate distance and angle
        let delta = end_point - start_point;
        let distance = delta.distance(vec2![0., 0.]);
        let angle_radians = delta.y.atan2(delta.x);
        let angle_degrees = angle_radians.to_degrees();

        // Set up paint properties
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_style(skia_safe::PaintStyle::Stroke);
        paint.set_stroke_width(OUTLINE_STROKE_THICKNESS * (1. / factor));

        // Line from start_point to mouse_position
        Cut::draw_line(i, v, canvas, &self.start_point, &self.end_point);

        // Horizontal line (unit vector guided)
        let mut path = Path::new();
        let end_point_horizontal = vec2!(start_point.x + distance, start_point.y);
        path.rewind(); // Clear the path to reuse it
        path.move_to(start_point.to_skia_point());
        path.line_to(end_point_horizontal.to_skia_point());
        canvas.draw_path(&path, &paint);

        // Arc illustrating the angle
        path.rewind(); // Clear the path to reuse it
        path.move_to(end_point_horizontal.to_skia_point());

        // Define the bounding box for the circle, centered on start_point
        let bounding_box = Rect::from_xywh(
            (start_point.x - distance) as f32,
            (start_point.y - distance) as f32,
            2.0 * distance as f32,
            2.0 * distance as f32,
        );

        // Now draw the arc
        path.arc_to(&bounding_box, 0.0, angle_degrees as f32, false);
        canvas.draw_path(&path, &paint);

        // Display the overall distance:
        // Choose a position to display the distance at the end of the line, slightly offset from the endpoint.
        let offset = 20.0;
        let adjusted_offset = if delta.x >= 0.0 { offset } else { -offset }; // Offset to the right if dx is positive, else to the left
        let distance_text_position = (
            end_point.x as f32 + adjusted_offset / factor,
            end_point.y as f32,
        );

        // Use draw_text function to display the distance at the chosen position
        self.draw_text(
            canvas,
            &format!("{:.2}", distance),
            distance_text_position,
            0.0,
            16.,
            factor,
        );

        // Display the angle beneath the horizontal line
        let text_position = vec2!(start_point.x + distance / 2.0, start_point.y + 10.0);
        self.draw_text(
            canvas,
            &format!("{:.2}°", angle_degrees),
            text_position.into(),
            0.0,
            16.,
            factor,
        );

        let intersections = Cut::find_intersections(&self.start_point, &self.end_point, v);
        self.draw_intersection_distances(canvas, intersections.as_slice(), factor);

        if intersections.len() > 1 {
            let first_intersection = intersections.first().unwrap();
            let last_intersection = intersections.last().unwrap();
            let fcoords = (
                first_intersection.coords.0 as f32,
                first_intersection.coords.1 as f32,
            );
            let lcoords = (
                last_intersection.coords.0 as f32,
                last_intersection.coords.1 as f32,
            );
            self.draw_bracket(canvas, fcoords, lcoords, -20.0, factor);
        }
    }

    fn draw_text(
        &self,
        canvas: &Canvas,
        text: &str,
        position: (f32, f32),
        angle: f32,
        size: f32,
        factor: f32,
    ) {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        let mut font = Font::default();
        font.set_size(size / factor);

        // Calculate text width and adjust the starting position to center the text
        let text_bounds = font.measure_str(text, Some(&paint));
        let adjusted_position = (
            position.0 - text_bounds.1.width() / 2.0,
            position.1 + text_bounds.1.height() / 2.0, // Adjust for vertical centering
        );

        // Create & apply rotation matrix
        let mut transform_matrix = Matrix::new_identity();
        transform_matrix.pre_translate(Point::from(adjusted_position));
        transform_matrix.pre_scale((1.0, -1.0), None); // Flip vertically
        transform_matrix.pre_rotate(angle, None); // Apply rotation
        canvas.save();
        canvas.concat(&transform_matrix); // Concatenate with existing matrix

        canvas.draw_str(text, Point::from((0., 0.)), &font, &paint);

        // Reset matrix to identity after drawing to avoid affecting other draws
        canvas.restore();
    }

    fn draw_intersection_distances(
        &self,
        canvas: &Canvas,
        intersections: &[Intersection],
        factor: f32,
    ) {
        let mut paint = Paint::default();
        paint.set_color(MEASURE_STROKE);
        paint.set_anti_alias(true);

        let mut font = Font::default();
        font.set_size(16. / factor);

        for window in intersections.windows(2) {
            let start = Point::from((window[0].coords.0 as f32, window[0].coords.1 as f32));
            let end = Point::from((window[1].coords.0 as f32, window[1].coords.1 as f32));

            // Calculate distance
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let distance = (dx.powi(2) + dy.powi(2)).sqrt();

            // Find midpoint for the text
            let midpoint = ((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);

            let text_angle = dy.atan2(dx).to_degrees();

            // Draw the distance text
            let text = format!("{:.2}", distance);
            self.draw_text(canvas, &text, midpoint, -text_angle, 8., factor)
        }
    }

    fn draw_bracket(
        &self,
        canvas: &Canvas,
        start: (f32, f32),
        end: (f32, f32),
        offset: f32,
        factor: f32,
    ) {
        // Calculate the direction vector of the main line
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;

        // Calculate a normalized perpendicular vector
        let magnitude = (dx.powi(2) + dy.powi(2)).sqrt();
        let perp_dx = -dy / magnitude;
        let perp_dy = dx / magnitude;

        // Calculate the endpoints of the bracket on the parallel line
        let bracket_start = (
            start.0 + perp_dx * offset / factor,
            start.1 + perp_dy * offset / factor,
        );
        let bracket_end = (
            end.0 + perp_dx * offset / factor,
            end.1 + perp_dy * offset / factor,
        );

        // Set up paint properties for drawing
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_style(skia_safe::PaintStyle::Stroke);
        paint.set_stroke_width(OUTLINE_STROKE_THICKNESS * (1. / factor));

        // Draw the parallel line for the bracket
        let mut path = Path::new();
        path.move_to(Point::from(bracket_start));
        path.line_to(Point::from(bracket_end));
        canvas.draw_path(&path, &paint);

        path.rewind();
        path.move_to(Point::from(bracket_start));
        path.line_to(Point::from(start));
        path.move_to(Point::from(bracket_end));
        path.line_to(Point::from(end));
        canvas.draw_path(&path, &paint);

        // Calculate the distance between start and end intersections
        let distance = ((end.0 - start.0).powi(2) + (end.1 - start.1).powi(2)).sqrt();

        // Calculate the midpoint of the bracket for text placement
        let midpoint = (
            (bracket_start.0 + bracket_end.0) / 2.0,
            (bracket_start.1 + bracket_end.1) / 2.0,
        );

        // Calculate the text's offset position so it's not obstructed by the bracket line
        let text_offset = -30.; // Adjust this value as needed
        let text_position = (
            midpoint.0 + perp_dx * text_offset / factor,
            midpoint.1 + perp_dy * text_offset / factor,
        );

        // The rotation angle is based on dy and dx which were calculated initially
        let rotation_angle = dy.atan2(dx).to_degrees();

        // Use draw_text function with rotation to display the distance
        // at the offset position and aligned with the bracket
        self.draw_text(
            canvas,
            &format!("{:.2}", distance),
            text_position,
            -rotation_angle,
            12.,
            factor,
        );
    }
}

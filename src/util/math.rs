// Very basic math functions for paths, rects, and points.

use skulpin::skia_safe as sk;

/// Skia Contains trait doesn't recognize a point as being contained if the rectangle is drawn
/// backwards or upside-down. This corrects for that.
pub trait FlipIfRequired {
    fn flip_if_required(&mut self);
}

impl FlipIfRequired for sk::Rect {
    fn flip_if_required(&mut self) {
        if self.right < self.left {
            let l = self.left;
            let r = self.right;
            self.right = l;
            self.left = r;
        }

        if self.bottom < self.top {
            let b = self.bottom;
            let t = self.top;
            self.top = b;
            self.bottom = t;
        }
    }
}

/// Trait necessary as f32 is primitive
pub trait RoundFloat {
    fn fround(self, digits: u8) -> Self;
}

impl RoundFloat for f32 {
    fn fround(self, digits: u8) -> Self {
        (self * 100.).round() / 100.
    }
}

use glifparser::{Handle, Point, PointType};

trait FromHandle<T> {
    fn from_handle(h: Handle) -> Point<T>;
}

impl<T> FromHandle<T> for Point<T> {
    /// Warning: Only call this on an At(x, y) variant!
    fn from_handle(h: Handle) -> Point<T> {
        let mut ret = Point::new();
        match h {
            Handle::At(x, y) => {
                ret.x = x;
                ret.y = y;
            }
            Handle::Colocated => {
                panic!("Cannot convert a colocated floating handle to a Point<T>!");
            }
        }
        ret
    }
}

pub trait DeCasteljau<T> {
    fn midpoint(p1: &Point<T>, p2: &Point<T>, t: f32) -> Point<T>;
    fn de_casteljau(inp: (Point<T>, Point<T>), t: Option<f32>) -> (Point<T>, Point<T>, Point<T>);
}

impl<T> DeCasteljau<T> for Point<T> {
    fn midpoint(p1: &Point<T>, p2: &Point<T>, t: f32) -> Point<T> {
        let mut ret = Point::new();
        ret.x = (p1.x + p2.x) * t;
        ret.y = (p1.y + p2.y) * t;
        ret
    }

    fn de_casteljau(inp: (Point<T>, Point<T>), t: Option<f32>) -> (Point<T>, Point<T>, Point<T>) {
        let (mut p1, mut p2) = inp;
        let (h1, h2) = (Point::from_handle(p1.b), Point::from_handle(p2.a));
        let t = t.unwrap_or(0.5);
        let dcmp1 = Point::midpoint(&p1, &h1, t);
        let dcmp2 = Point::midpoint(&h1, &h2, t);
        let dcmp3 = Point::midpoint(&h2, &p2, t);
        let dcmp_1 = Point::midpoint(&dcmp1, &dcmp2, t);
        let dcmp_2 = Point::midpoint(&dcmp2, &dcmp3, t);
        let mut dcmp__ = Point::midpoint(&dcmp_1, &dcmp_2, t);

        p1.b = Handle::At(dcmp1.x, dcmp1.y);
        p2.a = Handle::At(dcmp3.x, dcmp3.y);
        dcmp__.a = Handle::At(dcmp_1.x, dcmp_1.y);
        dcmp__.b = Handle::At(dcmp_2.x, dcmp_2.y);

        (p1, dcmp__, p2)
    }
}

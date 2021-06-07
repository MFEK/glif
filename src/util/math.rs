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
    fn fround(self) -> Self;
}

impl RoundFloat for f32 {
    fn fround(self) -> Self {
        (self * 100.).round() / 100.
    }
}

use glifparser::{Contour, Handle, Outline, Point, PointData, PointType};

trait FromHandle<P> {
    fn from_handle(h: Handle) -> Point<P>;
}

impl<P: PointData> FromHandle<P> for Point<P> {
    /// Warning: Only call this on an At(x, y) variant!
    fn from_handle(h: Handle) -> Point<P> {
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

// TODO: Implement this trait on the `data` members of ContourOperations.
pub trait ReverseContours {
    fn reverse_contours(self) -> Self;
}

impl<PD: PointData> ReverseContours for Contour<PD> {
    fn reverse_contours(self) -> Contour<PD> {
        let mut new_c = Contour::with_capacity(self.len());
        let open_contour = self.first().unwrap().ptype == PointType::Move && self.len() > 1;
        // This is necessary because although Rev and Chain both implement Iterator, they're
        // incompatible types. So, we need to make a mutable reference to a trait object.
        let (mut iter_t1, mut iter_t2);
        let iter: &mut dyn Iterator<Item = &Point<PD>> = if !open_contour {
            iter_t1 = self[0..1].iter().chain(self[1..].iter().rev());
            &mut iter_t1
        } else {
            iter_t2 = self.iter().rev();
            &mut iter_t2
        };

        // Considering the contour in reversed order, reverse a/b and point order.
        for p in iter {
            let mut new_p = p.clone();
            // a is next, b is prev
            let a = p.a;
            let b = p.b;
            new_p.a = b;
            new_p.b = a;
            new_p.ptype = if a != Handle::Colocated { PointType::Curve } else { PointType::Line };
            new_c.push(new_p);
        }

        // Considering only open contours post-reversal, reverse which point is the Move.
        if open_contour {
            new_c[0].ptype = PointType::Move;
            let c_len = new_c.len();
            let ptype = match new_c[c_len-2].a { Handle::At(_, _) => { PointType::Curve }, Handle::Colocated => { PointType::Line }};
            new_c[c_len-1].ptype = ptype;
        }

        new_c
    }
}

impl<PD: PointData> ReverseContours for Outline<PD> {
    fn reverse_contours(self) -> Outline<PD> {
        let mut ret = Outline::new();

        for c in self.iter() {
            let new_c = c.clone().reverse_contours();
            ret.push(new_c);
        }

        ret
    }
}

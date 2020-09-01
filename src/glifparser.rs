use crate::util;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PointType {
    Undefined,
    Move,
    Curve,
    QCurve,
    Line,
    OffCurve,
} // Undefined used by new(), shouldn't appear in Point structs

#[derive(Debug, Copy, Clone)]
pub enum AnchorType {
    Undefined,
    Mark,
    Base,
    MarkMark,
    MarkBase,
} // Undefined used everywhere for now as getting type requires parsing OpenType features, which we will be using nom to do since I have experience w/it.

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Handle {
    Colocated,
    At(f32, f32),
}

impl From<Option<&GlifPoint>> for Handle {
    fn from(point: Option<&GlifPoint>) -> Handle {
        match point {
            Some(p) => Handle::At(p.x, p.y),
            None => Handle::Colocated,
        }
    }
}

// A "close to the source <point>"
#[derive(Debug)]
struct GlifPoint {
    x: f32,
    y: f32,
    smooth: bool,
    name: Option<String>,
    ptype: PointType,
}

impl GlifPoint {
    fn new() -> GlifPoint {
        GlifPoint {
            x: 0.,
            y: 0.,
            ptype: PointType::Undefined,
            smooth: false,
            name: None,
        }
    }
}

type GlifContour = Vec<GlifPoint>;
type GlifOutline = Vec<GlifContour>;

// A Skia-friendly point
#[derive(Debug, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub a: Handle,
    pub b: Handle,
    name: Option<String>,
    pub ptype: PointType,
}

pub enum WhichHandle {
    A,
    B,
}
impl Point {
    pub fn new() -> Point {
        Point {
            x: 0.,
            y: 0.,
            a: Handle::Colocated,
            b: Handle::Colocated,
            ptype: PointType::Undefined,
            name: None,
        }
    }

    pub fn handle_or_colocated(
        &self,
        which: WhichHandle,
        transform_x: fn(f32) -> f32,
        transform_y: fn(f32) -> f32,
    ) -> (f32, f32) {
        let handle = match which {
            WhichHandle::A => self.a,
            WhichHandle::B => self.b,
        };
        match handle {
            Handle::At(x, y) => (transform_x(x), transform_y(y)),
            Handle::Colocated => (transform_x(self.x), transform_y(self.y)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Anchor {
    pub x: f32,
    pub y: f32,
    pub class: String,
    pub r#type: AnchorType,
}

impl Anchor {
    pub fn new() -> Anchor {
        Anchor {
            x: 0.,
            y: 0.,
            r#type: AnchorType::Undefined,
            class: String::new(),
        }
    }
}

pub type Contour = Vec<Point>;
pub type Outline = Vec<Contour>;

#[derive(Debug, PartialEq)]
pub enum Codepoint {
    Hex(char),
    Undefined,
}

#[derive(Debug)]
pub struct Glif {
    pub outline: Option<Outline>,
    pub anchors: Option<Vec<Anchor>>,
    pub width: u64,
    pub unicode: Codepoint,
    pub name: String,
    pub format: u8, // we only understand 2
}

extern crate xmltree;
use std::collections::VecDeque;
use std::error::Error;
use std::fs;

fn parse_anchor(anchor_el: xmltree::Element) -> Result<Anchor, &'static str> {
    Err("Unimplemented")
}

fn parse_point_type(pt: Option<&str>) -> PointType {
    match pt {
        Some("move") => PointType::Move,
        Some("line") => PointType::Line,
        Some("qcurve") => PointType::QCurve,
        Some("curve") => PointType::Curve,
        _ => PointType::OffCurve,
    }
}

pub fn read_ufo_glif(glif: &str) -> Glif {
    let mut glif = xmltree::Element::parse(glif.as_bytes()).expect("Invalid XML");
    let mut ret = Glif {
        outline: None,
        anchors: None,
        width: 0,
        unicode: Codepoint::Undefined,
        name: String::new(),
        format: 2,
    };
    assert_eq!(glif.name, "glyph", "Root element not <glyph>");
    assert_eq!(
        glif.attributes
            .get("format")
            .expect("<glyph> has no format"),
        "2",
        "<glyph> format not 2"
    );
    ret.name = glif
        .attributes
        .get("name")
        .expect("<glyph> has no name")
        .clone();
    let advance = glif
        .take_child("advance")
        .expect("<glyph> missing <advance> child");
    let unicode = glif.take_child("unicode");
    ret.width = advance
        .attributes
        .get("width")
        .expect("<advance> has no width")
        .parse()
        .expect("<advance> width not int");
    match unicode {
        Some(unicode) => {
            let unicodehex = unicode
                .attributes
                .get("hex")
                .expect("<unicode> has no width");
            ret.unicode = Codepoint::Hex(
                char::from_u32(u32::from_str_radix(unicodehex, 16).expect("<unicode> hex not int"))
                    .expect("<unicode> char conversion failed"),
            );
        }
        None => {
            ret.unicode = Codepoint::Undefined;
        }
    }

    let mut anchors: Vec<Anchor> = Vec::new();

    while let Some(anchor_el) = glif.take_child("anchor") {
        let mut anchor = Anchor::new();

        anchor.x = anchor_el
            .attributes
            .get("x")
            .expect("<anchor> missing x")
            .parse()
            .expect("<anchor> x not float");
        anchor.y = anchor_el
            .attributes
            .get("y")
            .expect("<anchor> missing y")
            .parse()
            .expect("<anchor> y not float");
        anchor.class = anchor_el
            .attributes
            .get("name")
            .expect("<anchor> missing class")
            .clone();
        anchors.push(anchor);
    }

    let mut goutline: GlifOutline = Vec::new();

    let mut outline_el = glif.take_child("outline");

    if outline_el.is_some() {
        let mut outline_elu = outline_el.unwrap();
        while let Some(mut contour_el) = outline_elu.take_child("contour") {
            let mut gcontour: GlifContour = Vec::new();
            while let Some(point_el) = contour_el.take_child("point") {
                let mut gpoint = GlifPoint::new();

                gpoint.x = point_el
                    .attributes
                    .get("x")
                    .expect("<point> missing x")
                    .parse()
                    .expect("<point> x not float");
                gpoint.y = point_el
                    .attributes
                    .get("y")
                    .expect("<point> missing y")
                    .parse()
                    .expect("<point> y not float");

                match point_el.attributes.get("name") {
                    Some(p) => gpoint.name = Some(p.clone()),
                    None => {}
                }

                gpoint.ptype =
                    parse_point_type(point_el.attributes.get("type").as_ref().map(|s| s.as_str()));

                gcontour.push(gpoint);
            }
            if gcontour.len() > 0 {
                goutline.push(gcontour);
            }
        }
    }

    let mut goiter = goutline.iter();

    let mut outline: Outline = Vec::new();

    let mut stack: VecDeque<&GlifPoint> = VecDeque::new();

    for gc in goutline.iter() {
        let mut contour: Contour = Vec::new();
        for gp in gc.iter() {
            match gp.ptype {
                PointType::OffCurve => {
                    stack.push_back(&gp);
                }
                _ => {
                    let h1 = stack.pop_front();
                    let h2 = stack.pop_front();
                    contour.last_mut().map(|p| p.a = Handle::from(h1));
                    contour.push(Point {
                        x: gp.x,
                        y: gp.y,
                        a: Handle::Colocated,
                        b: Handle::from(h2),
                        name: gp.name.clone(),
                        ptype: gp.ptype,
                    });
                }
            }
        }
        let h1 = stack.pop_front();
        let h2 = stack.pop_front();
        contour.last_mut().map(|p| p.a = Handle::from(h1));
        if contour.len() > 0 && contour[0].ptype != PointType::Move {
            contour.first_mut().map(|p| p.b = Handle::from(h2));
        }
        outline.push(contour);
    }

    if outline.len() > 0 {
        ret.outline = Some(outline);
    }

    if anchors.len() > 0 {
        ret.anchors = Some(anchors);
    }

    ret
}

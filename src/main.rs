use std::{
    io::{stdin, Read},
    ops::{Add, Div, Mul, Sub},
};
use usvg::{
    tiny_skia_path::{PathSegment, Point},
    Node, Options, Tree,
};

mod json;

#[derive(Debug, Clone, PartialEq, Default)]
struct SvgPath {
    id: String,
    segments: Vec<PathSegment>,
}

fn main() -> anyhow::Result<()> {
    let mut input = vec![];
    stdin().read_to_end(&mut input)?;
    let svg = Tree::from_data(input.as_slice(), &Options::default())?;
    let mut group_stack = vec![(svg.root(), "")];
    let mut paths = vec![];

    while let Some(top) = group_stack.pop() {
        let (group, id) = top;
        for child in group.children() {
            match child {
                Node::Group(group) => {
                    let cid = group.id();
                    let id = if cid.is_empty() { id } else { cid };
                    group_stack.push((group, id))
                }
                Node::Path(path) => {
                    let cid = path.id();
                    let id = if cid.is_empty() { id } else { cid };
                    if !path.is_visible() {
                        continue;
                    }
                    if path.fill().is_some() {
                        paths.push(SvgPath {
                            id: id.to_string(),
                            segments: path.data().segments().collect(),
                        });
                    }
                    if let Some(stroke) = path.stroke() {
                        if let Some(path_stroke) = path.data().stroke(&stroke.to_tiny_skia(), 1.0) {
                            paths.push(SvgPath {
                                id: id.to_string(),
                                segments: path_stroke.segments().collect(),
                            })
                        }
                    }
                    if path.is_visible() {
                        paths.push(SvgPath {
                            id: id.to_string(),
                            segments: path.data().segments().collect(),
                        })
                    }
                }
                Node::Image(_) | Node::Text(_) => {}
            }
        }
    }

    let mut prims = vec![];
    for path in paths {
        let mut prim: Option<PrimBuilder> = None;
        for segment in path.segments.into_iter() {
            match segment {
                PathSegment::MoveTo(p0) => {
                    if let Some(prim) = prim {
                        prims.push(prim.build());
                    }
                    prim = Some(PrimBuilder::new(p0.into()));
                }
                PathSegment::LineTo(p0) => {
                    if let Some(prim) = &mut prim {
                        prim.segments.push(Segment::Line(p0.into()));
                    }
                }
                PathSegment::QuadTo(p0, p1) => {
                    if let Some(prim) = &mut prim {
                        prim.order = prim.order.max(Order::Quad);
                        prim.segments.push(Segment::Quad(p0.into(), p1.into()))
                    }
                }
                PathSegment::CubicTo(p0, p1, p2) => {
                    if let Some(prim) = &mut prim {
                        prim.order = Order::Cube;
                        prim.segments
                            .push(Segment::Cube(p0.into(), p1.into(), p2.into()))
                    }
                }
                PathSegment::Close => {
                    if let Some(mut prim) = prim.take() {
                        prim.is_closed = true;
                        prims.push(prim.build());
                    }
                }
            }
        }
    }

    println!("{:#?}", prims);

    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
enum Order {
    #[default]
    Line,
    Quad,
    Cube,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
struct P(f32, f32);

impl From<Point> for P {
    fn from(value: Point) -> Self {
        Self(value.x, value.y)
    }
}

impl Sub for P {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Add for P {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Mul<f32> for P {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}

impl Div<f32> for P {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs, self.1 / rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Segment {
    Line(P),
    Quad(P, P),
    Cube(P, P, P),
}

const R13: f32 = 1.0 / 3.0;
const R23: f32 = 2.0 / 3.0;

impl Segment {
    pub fn to_cube(self, p0: P) -> [P; 3] {
        match self {
            Segment::Line(p1) => {
                let d = p1 - p0;
                [p0 + d * R13, p0 + d * R23, p1]
            }
            Segment::Quad(p1, p2) => [p0 + (p1 - p0) * R23, p2 + (p1 - p2) * R23, p2],
            Segment::Cube(p1, p2, p3) => [p1, p2, p3],
        }
    }

    pub fn to_quad(self, p0: P) -> [P; 2] {
        match self {
            Segment::Line(p1) => [(p0 + p1) / 2.0, p1],
            Segment::Quad(p1, p2) => [p1, p2],
            Segment::Cube(_, _, _) => panic!("Can't convert cube to quad"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
struct PrimBuilder {
    order: Order,
    is_closed: bool,
    start: P,
    segments: Vec<Segment>,
}

impl PrimBuilder {
    pub fn new(start: P) -> Self {
        Self {
            start,
            ..Default::default()
        }
    }

    pub fn build(self) -> Prim {
        let mut points = vec![self.start];
        match self.order {
            Order::Line => {
                for segment in self.segments {
                    let Segment::Line(p0) = segment else {
                        panic!("Expected a line");
                    };
                    points.push(p0);
                }
            }
            Order::Quad => {
                for segment in self.segments {
                    points.extend(segment.to_quad(*points.last().unwrap()));
                }
            }
            Order::Cube => {
                for segment in self.segments {
                    points.extend(segment.to_cube(*points.last().unwrap()));
                }
            }
        }
        if self.is_closed {
            points.pop();
        }
        Prim {
            order: self.order,
            is_closed: self.is_closed,
            points,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
struct Prim {
    order: Order,
    is_closed: bool,
    points: Vec<P>,
}

// [
// 	[
// 		"type","BezierCurve"
// 	],
// 	[
// 		"vertex",[104,105,106,107,108,109,110,111,112,113,114,115],
// 		"closed",true,
// 		"basis",[
// 			"type","Bezier",
// 			"order",4,
// 			"knots",[0,0.25,0.5,0.75,1]
// 		]
// 	]
// ],

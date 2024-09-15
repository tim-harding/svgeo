use std::{
    default,
    io::{stdin, Read},
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

    // Only support quad + closed to begin with
    let mut prims = vec![];
    for path in paths {
        let mut prim = None;
        for segment in path.segments.into_iter() {
            match segment {
                PathSegment::MoveTo(p0) => {
                    if let Some(prim) = prim {
                        prims.push(prim);
                    }
                    prim = Some(Prim::new(p0.into()));
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
                        prims.push(prim);
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Segment {
    Line(P),
    Quad(P, P),
    Cube(P, P, P),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
struct Prim {
    start: P,
    order: Order,
    segments: Vec<Segment>,
    is_closed: bool,
}

impl Prim {
    pub fn new(start: P) -> Self {
        Self {
            start,
            ..Default::default()
        }
    }
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

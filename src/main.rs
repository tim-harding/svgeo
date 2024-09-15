use json::{Value, ValueVec};
use std::{
    io::{stdin, Read},
    ops::{Add, Div, Mul, Sub},
};
use usvg::{
    tiny_skia_path::{PathSegment, Point},
    Group, Node, Options, Transform, Tree,
};

mod json;

#[derive(Debug, Clone, PartialEq, Default)]
struct SvgPath {
    id: String,
    segments: Vec<PathSegment>,
}

struct GroupInfo<'a> {
    id: String,
    group: &'a Group,
    parent_transform: Transform,
}

fn main() -> anyhow::Result<()> {
    let mut input = vec![];
    stdin().read_to_end(&mut input)?;
    let svg = Tree::from_data(input.as_slice(), &Options::default())?;
    let mut group_stack = vec![GroupInfo {
        id: "".to_string(),
        parent_transform: Transform::identity(),
        group: svg.root(),
    }];
    let mut paths = vec![];

    while let Some(top) = group_stack.pop() {
        let GroupInfo {
            id,
            parent_transform,
            group,
        } = top;
        let transform = parent_transform.pre_concat(group.transform());
        for child in group.children() {
            let cid = child.id();
            let id = if cid.is_empty() {
                id.clone()
            } else {
                format!("{id}/{cid}")
            };

            match child {
                Node::Group(group) => group_stack.push(GroupInfo {
                    id,
                    group,
                    parent_transform: transform,
                }),

                Node::Path(path) => {
                    if !path.is_visible() {
                        continue;
                    }

                    let Some(path_tx) = path.data().clone().transform(transform) else {
                        continue;
                    };

                    if path.fill().is_some() {
                        paths.push(SvgPath {
                            id: id.clone(),
                            segments: path_tx.segments().collect(),
                        });
                    }

                    if let Some(stroke) = path.stroke() {
                        if let Some(path_stroke) = path_tx.stroke(&stroke.to_tiny_skia(), 1.0) {
                            paths.push(SvgPath {
                                id,
                                segments: path_stroke.segments().collect(),
                            })
                        }
                    }
                }

                Node::Image(_) | Node::Text(_) => {}
            }
        }
    }

    let mut prims = vec![];
    for path in paths {
        let SvgPath { id, segments } = path;
        let mut prim: Option<PrimBuilder> = None;
        for segment in segments.into_iter() {
            match segment {
                PathSegment::MoveTo(p0) => {
                    if let Some(prim) = prim {
                        prims.push(prim.build(id.clone()));
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
                        prims.push(prim.build(id.clone()));
                    }
                }
            }
        }
    }

    let json = prims_to_json(prims);
    let s = json.to_string();
    println!("{s}");

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

    pub fn build(self, id: String) -> Prim {
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
            id,
            order: self.order,
            is_closed: self.is_closed,
            points,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
struct Prim {
    id: String,
    order: Order,
    is_closed: bool,
    points: Vec<P>,
}

fn prims_to_json(prims: Vec<Prim>) -> Value {
    let point_count = prims.iter().fold(0, |acc, prim| acc + prim.points.len());
    let indices = ValueVec((0..point_count as i64).map(|i| i.into()).collect());

    let points = ValueVec(
        prims
            .iter()
            .flat_map(|prim| prim.points.iter())
            .map(|p| {
                Value::from(ValueVec(
                    [p.0, p.1, 0.0].into_iter().map(Value::from).collect(),
                ))
            })
            .collect(),
    );

    let mut prim_i = 0;
    let mut primitives = vec![];
    let mut prim_ids = vec![];
    let prim_id_indices = ValueVec((0..prims.len()).map(Value::from).collect());
    for prim in prims.iter() {
        prim_ids.push(Value::from(prim.id.clone()));
        let value = Value::from(match prim.order {
            Order::Line => value_vec![
                value_vec!["type", "PolygonCurve_run"],
                value_vec![
                    "startvertex",
                    prim_i,
                    "nprimitives",
                    1,
                    "nvertices",
                    value_vec![prim.points.len()]
                ]
            ],

            Order::Cube | Order::Quad => {
                let order = match prim.order {
                    Order::Line => 2,
                    Order::Quad => 3,
                    Order::Cube => 4,
                };
                value_vec![
                    value_vec!["type", "BezierCurve"],
                    value_vec![
                        "vertex",
                        ValueVec(
                            (prim_i..prim_i + prim.points.len())
                                .map(Value::from)
                                .collect()
                        ),
                        "closed",
                        prim.is_closed,
                        "basis",
                        value_vec![
                            "type",
                            "Bezier",
                            "order",
                            order,
                            "knots",
                            ValueVec(
                                (0..prim.points.len() / (order - 1) + 1)
                                    .map(Value::from)
                                    .collect()
                            )
                        ]
                    ]
                ]
            }
        });

        primitives.push(value);
        prim_i += prim.points.len();
    }

    value_vec![
        "fileversion",
        "20.5.332",
        "hasindex",
        false,
        "pointcount",
        point_count,
        "vertexcount",
        point_count,
        "primitivecount",
        prims.len(),
        "info",
        value_obj! {},
        "topology",
        value_vec!["pointref", value_vec!["indices", indices]],
        "attributes",
        value_vec![
            "pointattributes",
            value_vec![value_vec![
                value_vec![
                    "scope",
                    "public",
                    "type",
                    "numeric",
                    "name",
                    "P",
                    "options",
                    value_obj! {
                        "type";value_obj!{
                            "type";"string",
                            "value";"point"
                        }
                    }
                ],
                value_vec![
                    "size",
                    3,
                    "storage",
                    "fpreal32",
                    "defaults",
                    value_vec!["size", 1, "storage", "fpreal64", "values", value_vec![0]],
                    "values",
                    value_vec!["size", 3, "storage", "fpreal32", "tuples", points]
                ]
            ]],
            "primitiveattributes",
            value_vec![value_vec![
                value_vec![
                    "scope",
                    "public",
                    "type",
                    "string",
                    "name",
                    "name",
                    "options",
                    value_obj! {}
                ],
                value_vec![
                    "size",
                    1,
                    "storage",
                    "int32",
                    "strings",
                    ValueVec(prim_ids),
                    "indices",
                    value_vec![
                        "size",
                        1,
                        "storage",
                        "int32",
                        "arrays",
                        value_vec![prim_id_indices]
                    ]
                ]
            ]]
        ],
        "primitives",
        ValueVec(primitives)
    ]
    .into()
}

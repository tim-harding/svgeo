use std::io::{stdin, Read};
use usvg::{tiny_skia_path::PathSegment, Group, Node, Options, Tree};

mod json;

struct Point(f32, f32, f32);

struct Primitive {
    id: String,
    segments: Vec<PathSegment>,
}

fn main() -> anyhow::Result<()> {
    let mut input = vec![];
    stdin().read_to_end(&mut input)?;
    let svg = Tree::from_data(input.as_slice(), &Options::default())?;
    let primitives = iter_primitives(svg.root());
    Ok(())
}

fn iter_primitives<I>(group: &Group) -> I
where
    I: Iterator<Item = Primitive>,
{
    group
        .children()
        .into_iter()
        .map(|child| -> FlatMapIter<Primitive, I> {
            match child {
                Node::Group(group) => FlatMapIter::Multi(iter_primitives(group.as_ref())),
                Node::Path(path) => FlatMapIter::Single(path.is_visible().then(|| Primitive {
                    id: path.id().into(),
                    segments: path.data().segments().collect(),
                })),
                Node::Image(_) | Node::Text(_) => FlatMapIter::Single(None),
            }
        })
        .flatten()
}

enum FlatMapIter<T, I>
where
    I: Iterator<Item = T>,
{
    Single(Option<T>),
    Multi(I),
}

impl<T, I> Iterator for FlatMapIter<T, I>
where
    I: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            FlatMapIter::Single(single) => single.take(),
            FlatMapIter::Multi(multi) => multi.next(),
        }
    }
}

fn get_segments(group: &Group, segments: &mut Vec<PathSegment>) {
    for child in group.children() {
        match child {
            Node::Group(group) => get_segments(group, segments),
            Node::Path(path) => {
                // TODO: path.fill for color
                if !path.is_visible() {
                    continue;
                }
                segments.extend(path.data().segments());
            }
            Node::Image(_) | Node::Text(_) => {}
        }
    }
}

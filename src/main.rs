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
    let primitives: Vec<_> = iter_primitives(svg.root()).collect();
    Ok(())
}

fn iter_primitives(group: &Group) -> impl Iterator<Item = Primitive> + '_ {
    group.children().into_iter().flat_map(|child| match child {
        Node::Group(group) => IterPair::Left(iter_primitives(group.as_ref())),
        Node::Path(path) => IterPair::Right(
            path.is_visible()
                .then(|| Primitive {
                    id: path.id().into(),
                    segments: path.data().segments().collect(),
                })
                .into_iter(),
        ),
        Node::Image(_) | Node::Text(_) => IterPair::Right(None.into_iter()),
    })
}

enum IterPair<T, L, R>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
{
    Left(L),
    Right(R),
}

impl<T, L, R> Iterator for IterPair<T, L, R>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterPair::Left(left) => left.next(),
            IterPair::Right(right) => right.next(),
        }
    }
}

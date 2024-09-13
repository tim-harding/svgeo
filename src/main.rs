use std::io::{stdin, Read};
use usvg::{tiny_skia_path::PathSegment, Group, Node, Options, Tree};

fn main() -> anyhow::Result<()> {
    let mut input = vec![];
    stdin().read_to_end(&mut input)?;
    let svg = Tree::from_data(input.as_slice(), &Options::default())?;
    let mut segments = vec![];
    get_segments(svg.root(), &mut segments);
    Ok(())
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

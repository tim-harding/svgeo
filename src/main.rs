use std::io::{stdin, Read};
use usvg::{tiny_skia_path::PathSegment, Node, Options, Tree};

//mod json;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
struct Point(f32, f32, f32);

#[derive(Debug, Clone, PartialEq, Default)]
struct Primitive {
    id: String,
    segments: Vec<PathSegment>,
}

fn main() -> anyhow::Result<()> {
    let mut input = vec![];
    stdin().read_to_end(&mut input)?;
    let svg = Tree::from_data(input.as_slice(), &Options::default())?;
    let mut group_stack = vec![(svg.root(), "")];
    let mut prims = vec![];
    while let Some(top) = group_stack.pop() {
        let (group, id) = top;
        for child in group.children().into_iter() {
            match child {
                Node::Group(group) => {
                    let cid = group.id();
                    let id = if cid == "" { id } else { cid };
                    group_stack.push((group, id))
                }
                Node::Path(path) => {
                    let cid = path.id();
                    let id = if cid == "" { id } else { cid };
                    if path.is_visible() {
                        prims.push(Primitive {
                            id: id.to_string(),
                            segments: path.data().segments().collect(),
                        })
                    }
                }
                Node::Image(_) | Node::Text(_) => {}
            }
        }
    }
    println!("{:#?}", prims);
    Ok(())
}

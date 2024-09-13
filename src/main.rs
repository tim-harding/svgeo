use std::io::{stdin, Read};
use usvg::{Options, Tree};

fn main() -> anyhow::Result<()> {
    let mut input = vec![];
    stdin().read_to_end(&mut input)?;
    let svg = Tree::from_data(input.as_slice(), &Options::default());
    Ok(())
}

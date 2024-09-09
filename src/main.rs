mod buffer;
mod editor;
mod event;

#[macro_use]
extern crate crossterm;

use clap::Parser;

#[derive(clap::Parser, Debug)]
#[command(about = "A terminal text editor", version = "0.1.0", author = "qwixck")]
struct Args {
    /// A path to a file you want edit
    path: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let buffer = buffer::Buffer::new(args.path)?;
    let mut editor = editor::Editor::new(buffer)?;
    editor.run()?;

    Ok(())
}

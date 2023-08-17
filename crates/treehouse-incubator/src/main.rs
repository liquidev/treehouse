use tree_html::HtmlGenerator;

mod tree_html;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("treehouse parsing error: {0}")]
    Parse(#[from] treehouse_format::ParseError),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = std::fs::remove_dir_all("target/site");
    std::fs::create_dir_all("target/site")?;

    let root_file = std::fs::read_to_string("content/tree/root.tree")?;

    let mut parser = treehouse_format::Parser {
        input: &root_file,
        position: 0,
    };
    let mut generator = HtmlGenerator::default();
    while let Some(branch) = parser.next_branch()? {
        for _ in 0..branch.indent_level {
            print!(" ");
        }
        println!(
            "{} {:?}",
            branch.kind.char(),
            &root_file[branch.content.clone()]
        );
        generator.add(&root_file, &branch);
    }
    std::fs::write(
        "target/site/index.html",
        format!(
            "<!DOCTYPE html><html><head></head><body>{}</body></html>",
            generator.finish()
        ),
    )?;

    Ok(())
}

use codespan_reporting::{
    diagnostic::{Diagnostic, Label, LabelStyle, Severity},
    files::SimpleFile,
    term::termcolor::{ColorChoice, StandardStream},
};
use treehouse_format::{
    ast::{Branch, Roots},
    pull::Parser,
};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("treehouse parsing error: {0}")]
    Parse(#[from] treehouse_format::ParseError),
}

fn print_branch(branch: &Branch, source: &str) {
    fn inner(branch: &Branch, source: &str, indent_level: usize) {
        for _ in 0..indent_level {
            print!("  ");
        }
        println!(
            "{} {:?}",
            branch.kind.char(),
            &source[branch.content.clone()]
        );
        for child in &branch.children {
            inner(child, source, indent_level + 1);
        }
    }
    inner(branch, source, 0);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = std::fs::remove_dir_all("target/site");
    std::fs::create_dir_all("target/site")?;

    let root_file = std::fs::read_to_string("content/tree/root.tree")?;
    let parse_result = Roots::parse(&mut Parser {
        input: &root_file,
        position: 0,
    });

    match parse_result {
        Ok(roots) => {
            for root in &roots.branches {
                print_branch(root, &root_file);
            }
        }
        Err(error) => {
            let writer = StandardStream::stderr(ColorChoice::Auto);
            let config = codespan_reporting::term::Config::default();
            let files = SimpleFile::new("root.tree", &root_file);
            let diagnostic = Diagnostic {
                severity: Severity::Error,
                code: None,
                message: error.kind.to_string(),
                labels: vec![Label {
                    style: LabelStyle::Primary,
                    file_id: (),
                    range: error.range,
                    message: String::new(),
                }],
                notes: vec![],
            };
            codespan_reporting::term::emit(&mut writer.lock(), &config, &files, &diagnostic)?;
        }
    }

    // let mut parser = treehouse_format::Parser {
    //     input: &root_file,
    //     position: 0,
    // };
    // let mut generator = HtmlGenerator::default();
    // while let Some(branch) = parser.next_branch()? {
    //     for _ in 0..branch.indent_level {
    //         print!(" ");
    //     }
    //     println!(
    //         "{} {:?}",
    //         branch.kind.char(),
    //         &root_file[branch.content.clone()]
    //     );
    //     generator.add(&root_file, &branch);
    // }
    // std::fs::write(
    //     "target/site/index.html",
    //     format!(
    //         "<!DOCTYPE html><html><head></head><body>{}</body></html>",
    //         generator.finish()
    //     ),
    // )?;

    Ok(())
}

use std::{ffi::OsStr, path::Path};

use anyhow::Context;
use treehouse_format::ast::{Branch, Roots};
use walkdir::WalkDir;

use crate::{
    parse::parse_tree_with_diagnostics,
    state::{Source, Treehouse},
};

use super::WcArgs;

fn wc_branch(source: &str, branch: &Branch) -> usize {
    let word_count = source[branch.content.clone()].split_whitespace().count();
    word_count
        + branch
            .children
            .iter()
            .map(|branch| wc_branch(source, branch))
            .sum::<usize>()
}

fn wc_roots(source: &str, roots: &Roots) -> usize {
    roots
        .branches
        .iter()
        .map(|branch| wc_branch(source, branch))
        .sum()
}

pub fn wc_cli(content_dir: &Path, mut wc_args: WcArgs) -> anyhow::Result<()> {
    if wc_args.paths.is_empty() {
        for entry in WalkDir::new(content_dir) {
            let entry = entry?;
            if entry.file_type().is_file() && entry.path().extension() == Some(OsStr::new("tree")) {
                wc_args.paths.push(entry.into_path());
            }
        }
    }

    let mut treehouse = Treehouse::new();

    let mut total = 0;

    for path in &wc_args.paths {
        let file = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read file to word count: {path:?}"))?;
        let path_without_ext = path.with_extension("");
        let utf8_filename = path_without_ext
            .strip_prefix(content_dir)
            .expect("paths should be rooted within the content directory")
            .to_string_lossy();

        let file_id = treehouse.add_file(utf8_filename.into_owned(), Source::Other(file));
        if let Ok(parsed) = parse_tree_with_diagnostics(&mut treehouse, file_id) {
            let source = treehouse.source(file_id);
            let word_count = wc_roots(source.input(), &parsed);
            println!("{word_count:>8} {}", treehouse.filename(file_id));
            total += word_count;
        }
    }

    println!("{total:>8} total");

    treehouse.report_diagnostics()?;

    Ok(())
}

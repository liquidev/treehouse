use std::ops::Range;

use anyhow::Context;
use treehouse_format::ast::Branch;

use crate::state::{FileId, Treehouse};

use super::{
    parse::{self, parse_toml_with_diagnostics, parse_tree_with_diagnostics},
    FixArgs,
};

struct Fix {
    range: Range<usize>,
    replacement: String,
}

#[derive(Default)]
struct State {
    fixes: Vec<Fix>,
}

fn dfs_fix_branch(treehouse: &mut Treehouse, file_id: FileId, state: &mut State, branch: &Branch) {
    let mut rng = rand::thread_rng();
    let ulid = ulid::Generator::new()
        .generate_with_source(&mut rng)
        .expect("failed to generate ulid for block"); // (wtf moment. you know how big the 80-bit combination space is?)

    let indent = " ".repeat(branch.indent_level);
    if let Some(attributes) = branch.attributes.clone() {
        // Scenario: Attributes need to be parsed as TOML and the id attribute has to be added into
        // the top-level table. Then we also need to pretty-print everything to match the right
        // indentation level.
        if let Ok(mut toml) =
            parse_toml_with_diagnostics(treehouse, file_id, attributes.data.clone())
        {
            if !toml.contains_key("id") {
                toml["id"] = toml_edit::value(ulid.to_string());
                toml.key_decor_mut("id")
                    .unwrap()
                    .set_prefix(" ".repeat(branch.indent_level + 2));
            }
            let mut toml_string = toml.to_string();

            // This is incredibly janky and barely works.
            let leading_spaces: usize = toml_string.chars().take_while(|&c| c == ' ').count();
            match leading_spaces {
                0 => toml_string.insert(0, ' '),
                1 => (),
                _ => toml_string.replace_range(0..leading_spaces - 1, ""),
            }

            let toml_string = fix_indent_in_generated_toml(&toml_string, branch.indent_level);

            state.fixes.push(Fix {
                range: attributes.data.clone(),
                replacement: toml_string,
            })
        }
    } else {
        // Scenario: No attributes at all.
        // In this case we can do a fast path where we generate the `% id = "whatever"` string
        // directly, not going through toml_edit.
        state.fixes.push(Fix {
            range: branch.kind_span.start..branch.kind_span.start,
            replacement: format!("% id = \"{ulid}\"\n{indent}"),
        });
    }

    // Then we fix child branches.
    for child in &branch.children {
        dfs_fix_branch(treehouse, file_id, state, child);
    }
}

fn fix_indent_in_generated_toml(toml: &str, min_indent_level: usize) -> String {
    let toml = toml.trim_end();

    let mut result = String::with_capacity(toml.len());

    for (i, line) in toml.lines().enumerate() {
        if line.is_empty() {
            result.push('\n');
        } else {
            let desired_line_indent_level = if i == 0 { 1 } else { min_indent_level + 2 };
            let leading_spaces: usize = line.chars().take_while(|&c| c == ' ').count();
            let needed_indentation = desired_line_indent_level.saturating_sub(leading_spaces);
            for _ in 0..needed_indentation {
                result.push(' ');
            }
            result.push_str(line);
            result.push('\n');
        }
    }

    for _ in 0..min_indent_level {
        result.push(' ');
    }

    result
}

pub fn fix_file(
    treehouse: &mut Treehouse,
    file_id: FileId,
) -> Result<String, parse::ErrorsEmitted> {
    parse_tree_with_diagnostics(treehouse, file_id).map(|roots| {
        let mut source = treehouse.get_source(file_id).to_owned();
        let mut state = State::default();

        for branch in &roots.branches {
            dfs_fix_branch(treehouse, file_id, &mut state, branch);
        }

        // Doing a depth-first search of the branches yields fixes from the beginning of the file
        // to its end. The most efficient way to apply all the fixes then is to reverse their order,
        // which lets us modify the source string in place because the fix ranges always stay
        // correct.
        for fix in state.fixes.iter().rev() {
            source.replace_range(fix.range.clone(), &fix.replacement);
        }

        source
    })
}

pub fn fix_file_cli(fix_args: FixArgs) -> anyhow::Result<()> {
    let utf8_filename = fix_args.file.to_string_lossy().into_owned();
    let file = std::fs::read_to_string(&fix_args.file).context("cannot read file to fix")?;

    let mut treehouse = Treehouse::new();
    let file_id = treehouse.files.add(utf8_filename, file);

    if let Ok(fixed) = fix_file(&mut treehouse, file_id) {
        if fix_args.apply {
            // Try to write the backup first. If writing that fails, bail out without overwriting
            // the source file.
            if let Some(backup_path) = fix_args.backup {
                std::fs::write(backup_path, treehouse.get_source(file_id))
                    .context("cannot write backup; original file will not be overwritten")?;
            }
            std::fs::write(&fix_args.file, fixed).context("cannot overwrite original file")?;
        } else {
            println!("{fixed}");
        }
    } else {
        treehouse.report_diagnostics()?;
    }

    Ok(())
}

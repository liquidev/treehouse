pub mod attributes;

use std::ops::Range;

use codespan_reporting::diagnostic::{Diagnostic, Label, LabelStyle, Severity};
use treehouse_format::{
    ast::{Branch, Roots},
    pull::BranchKind,
};

use crate::{
    state::{toml_error_to_diagnostic, FileId, TomlError, Treehouse},
    tree::attributes::{Attributes, Content},
};

#[derive(Debug, Default, Clone)]
pub struct SemaTree {
    branches: Vec<SemaBranch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemaBranchId(usize);

impl SemaTree {
    pub fn add_branch(&mut self, branch: SemaBranch) -> SemaBranchId {
        let id = self.branches.len();
        self.branches.push(branch);
        SemaBranchId(id)
    }

    pub fn branch(&self, id: SemaBranchId) -> &SemaBranch {
        &self.branches[id.0]
    }
}

#[derive(Debug, Clone)]
pub struct SemaRoots {
    pub branches: Vec<SemaBranchId>,
}

impl SemaRoots {
    pub fn from_roots(treehouse: &mut Treehouse, file_id: FileId, roots: Roots) -> Self {
        Self {
            branches: roots
                .branches
                .into_iter()
                .map(|branch| SemaBranch::from_branch(treehouse, file_id, branch))
                .collect(),
        }
    }
}

/// Analyzed branch.
#[derive(Debug, Clone)]
pub struct SemaBranch {
    pub file_id: FileId,

    pub indent_level: usize,
    pub raw_attributes: Option<treehouse_format::pull::Attributes>,
    pub kind: BranchKind,
    pub kind_span: Range<usize>,
    pub content: Range<usize>,

    pub html_id: String,
    pub attributes: Attributes,
    pub children: Vec<SemaBranchId>,
}

impl SemaBranch {
    pub fn from_branch(treehouse: &mut Treehouse, file_id: FileId, branch: Branch) -> SemaBranchId {
        let attributes = Self::parse_attributes(treehouse, file_id, &branch);

        let named_id = attributes.id.clone();
        let html_id = format!(
            "{}:{}",
            treehouse
                .tree_path(file_id)
                .expect("file should have a tree path"),
            attributes.id
        );

        let branch = Self {
            file_id,
            indent_level: branch.indent_level,
            raw_attributes: branch.attributes,
            kind: branch.kind,
            kind_span: branch.kind_span,
            content: branch.content,
            html_id,
            attributes,
            children: branch
                .children
                .into_iter()
                .map(|child| Self::from_branch(treehouse, file_id, child))
                .collect(),
        };
        let new_branch_id = treehouse.tree.add_branch(branch);

        if let Some(old_branch_id) = treehouse
            .branches_by_named_id
            .insert(named_id.clone(), new_branch_id)
        {
            let new_branch = treehouse.tree.branch(new_branch_id);
            let old_branch = treehouse.tree.branch(old_branch_id);

            treehouse.diagnostics.push(
                Diagnostic::warning()
                    .with_code("sema")
                    .with_message(format!("two branches share the same id `{}`", named_id))
                    .with_labels(vec![
                        Label {
                            style: LabelStyle::Primary,
                            file_id,
                            range: new_branch.kind_span.clone(),
                            message: String::new(),
                        },
                        Label {
                            style: LabelStyle::Primary,
                            file_id: old_branch.file_id,
                            range: old_branch.kind_span.clone(),
                            message: String::new(),
                        },
                    ]),
            )
        }

        new_branch_id
    }

    fn parse_attributes(treehouse: &mut Treehouse, file_id: FileId, branch: &Branch) -> Attributes {
        let source = treehouse.source(file_id);

        let mut successfully_parsed = true;
        let mut attributes = if let Some(attributes) = &branch.attributes {
            toml_edit::de::from_str(&source[attributes.data.clone()]).unwrap_or_else(|error| {
                treehouse
                    .diagnostics
                    .push(toml_error_to_diagnostic(TomlError {
                        message: error.message().to_owned(),
                        span: error.span(),
                        file_id,
                        input_range: attributes.data.clone(),
                    }));
                successfully_parsed = false;
                Attributes::default()
            })
        } else {
            Attributes::default()
        };
        let successfully_parsed = successfully_parsed;

        // Only check for attribute validity if the attributes were parsed successfully.
        if successfully_parsed {
            let attribute_warning_span = branch
                .attributes
                .as_ref()
                .map(|attributes| attributes.percent.clone())
                .unwrap_or(branch.kind_span.clone());

            // Check that every block has an ID.
            if attributes.id.is_empty() {
                attributes.id = format!("treehouse-missingno-{}", treehouse.next_missingno());
                treehouse.diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    code: Some("attr".into()),
                    message: "branch does not have an `id` attribute".into(),
                    labels: vec![Label {
                        style: LabelStyle::Primary,
                        file_id,
                        range: attribute_warning_span.clone(),
                        message: String::new(),
                    }],
                    notes: vec![
                        format!(
                            "note: a generated id `{}` will be used, but this id is unstable and will not persist across generations",
                            attributes.id
                        ),
                        format!("help: run `treehouse fix {}` to add missing ids to branches", treehouse.filename(file_id)),
                    ],
                });
            }

            // Check that link-type blocks are `+`-type to facilitate lazy loading.
            if let Content::Link(_) = &attributes.content {
                if branch.kind == BranchKind::Expanded {
                    treehouse.diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        code: Some("attr".into()),
                        message: "`content.link` branch is expanded by default".into(),
                        labels: vec![Label {
                            style: LabelStyle::Primary,
                            file_id,
                            range: branch.kind_span.clone(),
                            message: String::new(),
                        }],
                        notes: vec![
                            "note: `content.link` branches should normally be collapsed to allow for lazy loading".into(),
                        ],
                    });
                }
            }
        }
        attributes
    }
}

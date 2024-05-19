pub mod attributes;
pub mod mini_template;

use std::ops::Range;

use codespan_reporting::diagnostic::{Diagnostic, Label, LabelStyle, Severity};
use treehouse_format::{
    ast::{Branch, Roots},
    pull::BranchKind,
};

use crate::{
    config::Config,
    state::{toml_error_to_diagnostic, FileId, Source, TomlError, Treehouse},
    tree::attributes::{Attributes, Content},
};

use self::attributes::RootAttributes;

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
    pub attributes: RootAttributes,
    pub branches: Vec<SemaBranchId>,
}

impl SemaRoots {
    pub fn from_roots(
        treehouse: &mut Treehouse,
        config: &Config,
        file_id: FileId,
        roots: Roots,
    ) -> Self {
        Self {
            attributes: Self::parse_attributes(treehouse, config, file_id, &roots),
            branches: roots
                .branches
                .into_iter()
                .map(|branch| SemaBranch::from_branch(treehouse, file_id, branch))
                .collect(),
        }
    }

    fn parse_attributes(
        treehouse: &mut Treehouse,
        config: &Config,
        file_id: FileId,
        roots: &Roots,
    ) -> RootAttributes {
        let source = treehouse.source(file_id);

        let mut successfully_parsed = true;
        let mut attributes = if let Some(attributes) = &roots.attributes {
            toml_edit::de::from_str(&source.input()[attributes.data.clone()]).unwrap_or_else(
                |error| {
                    treehouse
                        .diagnostics
                        .push(toml_error_to_diagnostic(TomlError {
                            message: error.message().to_owned(),
                            span: error.span(),
                            file_id,
                            input_range: attributes.data.clone(),
                        }));
                    successfully_parsed = false;
                    RootAttributes::default()
                },
            )
        } else {
            RootAttributes::default()
        };
        let successfully_parsed = successfully_parsed;

        if successfully_parsed && attributes.title.is_empty() {
            attributes.title = match treehouse.source(file_id) {
                Source::Tree { tree_path, .. } => tree_path.clone(),
                _ => panic!("parse_attributes called for a non-.tree file"),
            }
        }

        if let Some(thumbnail) = &attributes.thumbnail {
            if thumbnail.alt.is_none() {
                treehouse.diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    code: Some("sema".into()),
                    message: "thumbnail without alt text".into(),
                    labels: vec![Label {
                        style: LabelStyle::Primary,
                        file_id,
                        range: roots.attributes.as_ref().unwrap().percent.clone(),
                        message: "".into(),
                    }],
                    notes: vec![
                        "note: alt text is important for people using screen readers".into(),
                        "help: add alt text using the thumbnail.alt key".into(),
                    ],
                })
            }

            if !config.pics.contains_key(&thumbnail.id) {
                treehouse.diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    code: Some("sema".into()),
                    message: format!(
                        "thumbnail picture with id '{}' does not exist",
                        thumbnail.id
                    ),
                    labels: vec![Label {
                        style: LabelStyle::Primary,
                        file_id,
                        range: roots.attributes.as_ref().unwrap().percent.clone(),
                        message: "".into(),
                    }],
                    notes: vec!["note: check your id for typos".into()],
                })
            }
        }

        attributes
    }
}

/// Analyzed branch.
#[derive(Debug, Clone)]
pub struct SemaBranch {
    pub file_id: FileId,

    pub indent_level: usize,
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
            toml_edit::de::from_str(&source.input()[attributes.data.clone()]).unwrap_or_else(
                |error| {
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
                },
            )
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

use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{bail, Context};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label, LabelStyle, Severity},
    files::Files as _,
};
use copy_dir::copy_dir;
use handlebars::{handlebars_helper, Handlebars};
use log::{debug, error, info};
use serde::Serialize;
use walkdir::WalkDir;

use crate::{
    config::{Config, ConfigDerivedData},
    fun::seasons::Season,
    html::{
        breadcrumbs::breadcrumbs_to_html,
        navmap::{build_navigation_map, NavigationMap},
        tree::branches_to_html,
    },
    import_map::ImportMap,
    include_static::IncludeStatic,
    parse::parse_tree_with_diagnostics,
    state::Source,
    static_urls::StaticUrls,
    tree::SemaRoots,
};

use crate::state::{FileId, Treehouse};

use super::Paths;

#[derive(Default)]
struct Generator {
    tree_files: Vec<PathBuf>,
}

struct Build {}

struct ParsedTree {
    tree_path: String,
    file_id: FileId,
    target_path: PathBuf,
}

#[derive(Serialize)]
struct Feed {
    branches: Vec<String>,
}

#[derive(Serialize)]
pub struct Page {
    pub title: String,
    pub thumbnail: Option<Thumbnail>,
    pub scripts: Vec<String>,
    pub styles: Vec<String>,
    pub breadcrumbs: String,
    pub tree_path: Option<String>,
    pub tree: String,
}

#[derive(Serialize)]
pub struct Thumbnail {
    pub url: String,
    pub alt: Option<String>,
}

#[derive(Serialize)]
struct StaticTemplateData<'a> {
    config: &'a Config,
    season: Option<Season>,
}

#[derive(Serialize)]
struct PageTemplateData<'a> {
    config: &'a Config,
    page: Page,
    feeds: &'a HashMap<String, Feed>,
    season: Option<Season>,
}

impl Generator {
    fn add_directory_rec(&mut self, directory: &Path) -> anyhow::Result<()> {
        for entry in WalkDir::new(directory) {
            let entry = entry?;
            if entry.path().extension() == Some(OsStr::new("tree")) {
                self.tree_files.push(entry.path().to_owned());
            }
        }
        Ok(())
    }

    fn register_template(
        handlebars: &mut Handlebars<'_>,
        treehouse: &mut Treehouse,
        name: &str,
        path: &Path,
    ) -> anyhow::Result<FileId> {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read template file {path:?}"))?;
        let file_id =
            treehouse.add_file(path.to_string_lossy().into_owned(), Source::Other(source));
        let source = treehouse.source(file_id);
        if let Err(error) = handlebars.register_template_string(name, source) {
            Self::wrangle_handlebars_error_into_diagnostic(
                treehouse,
                file_id,
                error.line_no,
                error.column_no,
                error.reason().to_string(),
            )?;
        }
        Ok(file_id)
    }

    fn wrangle_handlebars_error_into_diagnostic(
        treehouse: &mut Treehouse,
        file_id: FileId,
        line: Option<usize>,
        column: Option<usize>,
        message: String,
    ) -> anyhow::Result<()> {
        if let (Some(line), Some(column)) = (line, column) {
            let line_range = treehouse
                .files
                .line_range(file_id, line)
                .expect("file was added to the list");
            treehouse.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: Some("template".into()),
                message,
                labels: vec![Label {
                    style: LabelStyle::Primary,
                    file_id,
                    range: line_range.start + column..line_range.start + column + 1,
                    message: String::new(),
                }],
                notes: vec![],
            })
        } else {
            let file = treehouse.filename(file_id);
            bail!("template error in {file}: {message}");
        }
        Ok(())
    }

    fn parse_trees(
        &self,
        config: &Config,
        paths: &Paths<'_>,
    ) -> anyhow::Result<(Treehouse, Vec<ParsedTree>)> {
        let mut treehouse = Treehouse::new();
        let mut parsed_trees = vec![];

        for path in &self.tree_files {
            let utf8_filename = path.to_string_lossy();

            let tree_path = path.strip_prefix(paths.content_dir).unwrap_or(path);
            let target_path = if tree_path == OsStr::new("index.tree") {
                paths.target_dir.join("index.html")
            } else {
                paths.target_dir.join(tree_path).with_extension("html")
            };
            debug!("generating: {path:?} -> {target_path:?}");

            let source = match std::fs::read_to_string(path) {
                Ok(source) => source,
                Err(error) => {
                    treehouse.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        code: None,
                        message: format!("{utf8_filename}: cannot read file: {error}"),
                        labels: vec![],
                        notes: vec![],
                    });
                    continue;
                }
            };
            let tree_path = tree_path
                .with_extension("")
                .to_string_lossy()
                .replace('\\', "/");
            let file_id = treehouse.add_file(
                utf8_filename.into_owned(),
                Source::Tree {
                    input: source,
                    tree_path: tree_path.clone(),
                },
            );

            if let Ok(roots) = parse_tree_with_diagnostics(&mut treehouse, file_id) {
                let roots = SemaRoots::from_roots(&mut treehouse, config, file_id, roots);
                treehouse.roots.insert(tree_path.clone(), roots);
                parsed_trees.push(ParsedTree {
                    tree_path,
                    file_id,
                    target_path,
                });
            }
        }

        Ok((treehouse, parsed_trees))
    }

    fn generate_all_files(
        &self,
        treehouse: &mut Treehouse,
        config: &Config,
        paths: &Paths<'_>,
        navigation_map: &NavigationMap,
        parsed_trees: Vec<ParsedTree>,
    ) -> anyhow::Result<()> {
        let mut handlebars = Handlebars::new();
        let mut config_derived_data = ConfigDerivedData {
            image_sizes: Default::default(),
            static_urls: StaticUrls::new(
                // NOTE: Allow referring to generated static assets here.
                paths.target_dir.join("static"),
                format!("{}/static", config.site),
            ),
        };

        handlebars_helper!(cat: |a: String, b: String| a + &b);

        handlebars.register_helper("cat", Box::new(cat));
        handlebars.register_helper(
            "asset",
            Box::new(StaticUrls::new(
                paths.target_dir.join("static"),
                format!("{}/static", config.site),
            )),
        );
        handlebars.register_helper(
            "include_static",
            Box::new(IncludeStatic {
                // NOTE: Again, allow referring to generated static assets.
                // This is necessary for import maps, for whom the <src> attribute is not
                // currently supported.
                base_dir: paths.target_dir.join("static"),
            }),
        );

        let mut template_file_ids = HashMap::new();
        for entry in WalkDir::new(paths.template_dir) {
            let entry = entry.context("cannot read directory entry")?;
            let path = entry.path();
            if !entry.file_type().is_dir() && path.extension() == Some(OsStr::new("hbs")) {
                let relative_path = path
                    .strip_prefix(paths.template_dir)?
                    .to_string_lossy()
                    .into_owned()
                    .replace('\\', "/");
                let file_id =
                    Self::register_template(&mut handlebars, treehouse, &relative_path, path)?;
                template_file_ids.insert(relative_path, file_id);
            }
        }

        std::fs::create_dir_all(paths.template_target_dir)?;
        for (name, &file_id) in &template_file_ids {
            let filename = name.rsplit_once('/').unwrap_or(("", name)).1;
            if !filename.starts_with('_') {
                let templated_html = match handlebars.render(
                    name,
                    &StaticTemplateData {
                        config,
                        season: Season::current(),
                    },
                ) {
                    Ok(html) => html,
                    Err(error) => {
                        Self::wrangle_handlebars_error_into_diagnostic(
                            treehouse,
                            file_id,
                            error.line_no,
                            error.column_no,
                            error.desc,
                        )?;
                        continue;
                    }
                };
                std::fs::write(
                    paths.template_target_dir.join(name).with_extension("html"),
                    templated_html,
                )?;
            }
        }

        let mut feeds = HashMap::new();

        for parsed_tree in &parsed_trees {
            let roots = &treehouse.roots[&parsed_tree.tree_path];

            if let Some(feed_name) = &roots.attributes.feed {
                let mut feed = Feed {
                    branches: Vec::new(),
                };
                for &root in &roots.branches {
                    let branch = treehouse.tree.branch(root);
                    feed.branches.push(branch.attributes.id.clone());
                }
                feeds.insert(feed_name.to_owned(), feed);
            }
        }

        for parsed_tree in parsed_trees {
            let breadcrumbs = breadcrumbs_to_html(config, navigation_map, &parsed_tree.tree_path);

            let mut tree = String::new();
            // Temporarily steal the tree out of the treehouse.
            let roots = treehouse
                .roots
                .remove(&parsed_tree.tree_path)
                .expect("tree should have been added to the treehouse");
            branches_to_html(
                &mut tree,
                treehouse,
                config,
                &mut config_derived_data,
                paths,
                parsed_tree.file_id,
                &roots.branches,
            );

            let template_data = PageTemplateData {
                config,
                page: Page {
                    title: roots.attributes.title.clone(),
                    thumbnail: roots
                        .attributes
                        .thumbnail
                        .as_ref()
                        .map(|thumbnail| Thumbnail {
                            url: config.pic_url(&thumbnail.id),
                            alt: thumbnail.alt.clone(),
                        }),
                    scripts: roots.attributes.scripts.clone(),
                    styles: roots.attributes.styles.clone(),
                    breadcrumbs,
                    tree_path: treehouse
                        .tree_path(parsed_tree.file_id)
                        .map(|s| s.to_owned()),
                    tree,
                },
                feeds: &feeds,
                season: Season::current(),
            };
            let template_name = roots
                .attributes
                .template
                .clone()
                .unwrap_or_else(|| "_tree.hbs".into());

            treehouse.roots.insert(parsed_tree.tree_path, roots);

            let templated_html = match handlebars.render(&template_name, &template_data) {
                Ok(html) => html,
                Err(error) => {
                    Self::wrangle_handlebars_error_into_diagnostic(
                        treehouse,
                        template_file_ids[&template_name],
                        error.line_no,
                        error.column_no,
                        error.desc,
                    )?;
                    continue;
                }
            };

            std::fs::create_dir_all(
                parsed_tree
                    .target_path
                    .parent()
                    .expect("there should be a parent directory to generate files into"),
            )?;
            std::fs::write(parsed_tree.target_path, templated_html)?;
        }

        Ok(())
    }
}

pub fn generate(paths: &Paths<'_>) -> anyhow::Result<(Config, Treehouse)> {
    let start = Instant::now();

    info!("loading config");
    let mut config = Config::load(paths.config_file)?;
    config.site = std::env::var("TREEHOUSE_SITE").unwrap_or(config.site);
    config.autopopulate_emoji(&paths.static_dir.join("emoji"))?;
    config.autopopulate_pics(&paths.static_dir.join("pic"))?;
    config.load_syntaxes(&paths.static_dir.join("syntax"))?;

    info!("cleaning target directory");
    let _ = std::fs::remove_dir_all(paths.target_dir);
    std::fs::create_dir_all(paths.target_dir)?;

    info!("copying static directory to target directory");
    copy_dir(paths.static_dir, paths.target_dir.join("static"))?;

    info!("creating static/generated directory");
    std::fs::create_dir_all(paths.target_dir.join("static/generated"))?;

    info!("parsing tree");
    let mut generator = Generator::default();
    generator.add_directory_rec(paths.content_dir)?;
    let (mut treehouse, parsed_trees) = generator.parse_trees(&config, paths)?;

    // NOTE: The navigation map is a legacy feature that is lazy-loaded when fragment-based
    // navigation is used.
    // I couldn't be bothered with adding it to the import map since fragment-based navigation is
    // only used on very old links. Adding caching to the navigation map is probably not worth it.
    info!("generating navigation map");
    let navigation_map = build_navigation_map(&treehouse, "index");
    std::fs::write(
        paths.target_dir.join("navmap.js"),
        navigation_map.to_javascript(),
    )?;

    info!("generating import map");
    let import_map =
        ImportMap::generate(config.site.clone(), &config.build.javascript.import_roots);
    std::fs::write(
        paths.target_dir.join("static/generated/import-map.json"),
        serde_json::to_string_pretty(&import_map).context("could not serialize import map")?,
    )?;

    info!("generating standalone pages");
    generator.generate_all_files(
        &mut treehouse,
        &config,
        paths,
        &navigation_map,
        parsed_trees,
    )?;

    treehouse.report_diagnostics()?;

    let duration = start.elapsed();
    info!("generation done in {duration:?}");

    if !treehouse.has_errors() {
        Ok((config, treehouse))
    } else {
        bail!("generation errors occurred; diagnostics were emitted with detailed descriptions");
    }
}

pub fn regenerate_or_report_error(paths: &Paths<'_>) -> anyhow::Result<(Config, Treehouse)> {
    info!("regenerating site content");

    let result = generate(paths);
    if let Err(e) = &result {
        error!("{e:?}");
    }
    result
}

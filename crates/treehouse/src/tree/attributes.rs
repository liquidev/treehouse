use serde::{Deserialize, Serialize};

/// Top-level `%%` root attributes.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RootAttributes {
    /// Template to use for generating the page.
    /// Defaults to `_tree.hbs`.
    #[serde(default)]
    pub template: Option<String>,

    /// Title of the generated .html page.
    ///
    /// The page's tree path is used if empty.
    #[serde(default)]
    pub title: String,

    /// Summary of the generated .html page.
    #[serde(default)]
    pub description: Option<String>,

    /// ID of picture attached to the page, to be used as a thumbnail.
    #[serde(default)]
    pub thumbnail: Option<Picture>,

    /// Additional scripts to load into to the page.
    /// These are relative to the /static/js directory.
    #[serde(default)]
    pub scripts: Vec<String>,

    /// Additional styles to load into to the page.
    /// These are relative to the /static/css directory.
    #[serde(default)]
    pub styles: Vec<String>,

    /// When specified, branches coming from this root will be added to a _feed_ with the given name.
    /// Feeds can be read by Handlebars templates to generate content based on them.
    #[serde(default)]
    pub feed: Option<String>,
}

/// A picture reference.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Picture {
    /// ID of the picture.
    pub id: String,

    /// Optional alt text.
    #[serde(default)]
    pub alt: Option<String>,
}

/// Branch attributes.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct Attributes {
    /// Unique identifier of the branch.
    ///
    /// Note that this must be unique to the _entire_ site, not just a single tree.
    /// This is because trees may be embedded within each other using [`Content::Link`].
    #[serde(default)]
    pub id: String,

    /// Controls how the block should be presented.
    #[serde(default)]
    pub content: Content,

    /// Do not persist the branch in localStorage.
    #[serde(default)]
    pub do_not_persist: bool,

    /// Strings of extra CSS class names to include in the generated HTML.
    #[serde(default)]
    pub classes: Classes,

    /// Enable `mini_template` templating in this branch.
    #[serde(default)]
    pub template: bool,

    /// Publishing stage; if `Draft`, the branch is invisible unless treehouse is compiled in
    /// debug mode.
    #[serde(default)]
    pub stage: Stage,
}

/// Controls for block content presentation.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Content {
    /// Children are stored inline in the block. Nothing special happens.
    #[default]
    Inline,

    /// Link to another tree.
    ///
    /// When JavaScript is enabled, the tree's roots will be embedded inline into the branch and
    /// loaded lazily.
    ///
    /// Without JavaScript, the tree will be linked with an `<a>` element.
    ///
    /// The string provided as an argument is relative to the `content` root and should not contain
    /// any file extensions. For example, to link to `content/my-article.tree`,
    /// use `content.link = "my-article"`.
    ///
    /// Note that `Link` branches must not contain any children. If a `Link` branch does contain
    /// children, an `attribute`-type error is raised.
    Link(String),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct Classes {
    /// Classes to append to the branch itself (<li data-cast="b">).
    #[serde(default)]
    pub branch: String,

    /// Classes to append to the branch's <ul> element containing its children.
    #[serde(default)]
    pub branch_children: String,
}

/// Publish stage of a branch.
///
/// Draft branches are not included in release builds of treehouse. In debug builds, they are also
/// marked with an extra "draft" before the content.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub enum Stage {
    #[default]
    Public,
    Draft,
}

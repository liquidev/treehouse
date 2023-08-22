use serde::Deserialize;

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

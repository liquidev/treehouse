use std::{borrow::Cow, fmt::Write};

use crate::config::Config;

use super::{navmap::NavigationMap, EscapeAttribute};

pub fn breadcrumbs_to_html(
    config: &Config,
    navigation_map: &NavigationMap,
    tree_path: &str,
) -> String {
    let mut s = String::new();

    if let Some(path) = navigation_map.paths.get(tree_path) {
        for (i, element) in path.iter().enumerate() {
            // Skip the index because it's implied by the logo on the left.
            if element != "index" {
                s.push_str("<li class=\"breadcrumb\">");
                {
                    let short_element = path
                        .get(i - 1)
                        .map(|p| format!("{p}/"))
                        .and_then(|prefix| element.strip_prefix(prefix.as_str()).map(Cow::Borrowed))
                        .unwrap_or_else(|| Cow::Owned(format!("/{element}")));
                    write!(
                        s,
                        "<a href=\"{site}/{element}.html\">{short_element}</a>",
                        site = EscapeAttribute(&config.site),
                        element = EscapeAttribute(element)
                    )
                    .unwrap();
                }
                s.push_str("</li>");
            }
        }
    }

    s
}

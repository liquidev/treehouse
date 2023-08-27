use std::collections::HashMap;

use serde::Serialize;

use crate::{
    state::Treehouse,
    tree::{attributes::Content, SemaBranchId},
};

#[derive(Debug, Clone, Default, Serialize)]
pub struct NavigationMap {
    /// Tells you which pages need to be opened to get to the key.
    pub paths: HashMap<String, Vec<String>>,
}

impl NavigationMap {
    pub fn to_javascript(&self) -> String {
        format!(
            "export const navigationMap = {};",
            serde_json::to_string(&self.paths)
                .expect("serialization of the navigation map should not fail")
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct NavigationMapBuilder {
    stack: Vec<String>,
    navigation_map: NavigationMap,
}

impl NavigationMapBuilder {
    pub fn enter_tree(&mut self, tree: String) {
        self.stack.push(tree.clone());
        self.navigation_map.paths.insert(tree, self.stack.clone());
    }

    pub fn exit_tree(&mut self) {
        self.stack.pop();
    }

    pub fn finish(self) -> NavigationMap {
        self.navigation_map
    }
}

pub fn build_navigation_map(treehouse: &Treehouse, root_tree_path: &str) -> NavigationMap {
    let mut builder = NavigationMapBuilder::default();

    fn rec_branch(
        treehouse: &Treehouse,
        builder: &mut NavigationMapBuilder,
        branch_id: SemaBranchId,
    ) {
        let branch = treehouse.tree.branch(branch_id);
        if let Content::Link(linked) = &branch.attributes.content {
            rec_tree(treehouse, builder, linked);
        } else {
            for &child_id in &branch.children {
                rec_branch(treehouse, builder, child_id);
            }
        }
    }

    fn rec_tree(treehouse: &Treehouse, builder: &mut NavigationMapBuilder, tree_path: &str) {
        if let Some(roots) = treehouse.roots.get(tree_path) {
            // Pages can link to each other causing infinite recursion, so we need to handle that
            // case by skipping pages that already have been analyzed.
            if !builder.navigation_map.paths.contains_key(tree_path) {
                builder.enter_tree(tree_path.to_owned());
                for &branch_id in &roots.branches {
                    rec_branch(treehouse, builder, branch_id);
                }
                builder.exit_tree();
            }
        }
    }

    rec_tree(treehouse, &mut builder, root_tree_path);

    builder.finish()
}

use treehouse_format::Branch;

#[derive(Debug, Clone, Default)]
pub struct HtmlGenerator {
    buffer: String,
    indent_level_stack: Vec<usize>,
}

impl HtmlGenerator {
    pub fn add(&mut self, source: &str, branch: &Branch) {
        if Some(&branch.indent_level) > self.indent_level_stack.last() {
            self.indent_level_stack.push(branch.indent_level);
            self.buffer.push_str("<ul>");
        }
        while Some(&branch.indent_level) < self.indent_level_stack.last() {
            self.indent_level_stack.pop();
            self.buffer.push_str("</ul>");
        }
        self.buffer.push_str("<li>");
        self.buffer.push_str(&source[branch.content.clone()]);
        self.buffer.push_str("</li>");
    }

    pub fn finish(mut self) -> String {
        for _ in self.indent_level_stack.drain(..) {
            self.buffer.push_str("</ul>");
        }
        self.buffer
    }
}

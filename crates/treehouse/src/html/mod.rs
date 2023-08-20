use std::fmt::{self, Display, Write};

pub mod attributes;
mod markdown;
pub mod tree;

pub struct EscapeAttribute<'a>(&'a str);

impl<'a> Display for EscapeAttribute<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0.chars() {
            if c == '"' {
                f.write_str("&quot;")?;
            } else {
                f.write_char(c)?;
            }
        }
        Ok(())
    }
}

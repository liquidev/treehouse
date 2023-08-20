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

pub struct EscapeHtml<'a>(&'a str);

impl<'a> Display for EscapeHtml<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0.chars() {
            match c {
                '<' => f.write_str("&lt;")?,
                '>' => f.write_str("&gt;")?,
                '&' => f.write_str("&amp;")?,
                '\'' => f.write_str("&apos;")?,
                '"' => f.write_str("&quot;")?,
                _ => f.write_char(c)?,
            }
        }
        Ok(())
    }
}

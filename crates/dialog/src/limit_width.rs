/// Format a dialog message.
///
/// - Keeps line width under 96 characters.
///
/// # Known Issues
/// - Inconsistent line break handling.
pub fn limit_width<T: std::fmt::Display>(msg: T) -> String {
    const MAX_LINE_LENGTH: usize = 96;
    let mut formatted = String::new();
    let string = msg.to_string();
    let mut lines = string.trim().split('\n').peekable();

    while let Some(line) = lines.next() {
        let mut current_line_length = 0;
        for word in line.split_whitespace() {
            let space_needed = if current_line_length > 0 { 1 } else { 0 };
            if current_line_length + space_needed + word.len() > MAX_LINE_LENGTH {
                formatted.push('\n');
                current_line_length = 0;
            } else if current_line_length > 0 {
                formatted.push(' ');
                current_line_length += 1;
            }

            formatted.push_str(word);
            current_line_length += word.len();
        }
        formatted.push('\n');
        if lines.peek().is_some() && line.is_empty() {
            formatted.push('\n');
        }
    }

    formatted.retain(|c| !c.is_ascii_control() || c.is_ascii_control() && c == '\n');
    formatted
}

/// A trait with 1 function, `limit_width`, which is implemented for [`std::fmt::Display`].
pub trait LimitWidthExt<T> {
    /// Limit the width of an object compatible with [`std::fmt::Display`] to 96 characters.
    ///
    /// Intended to be used for formatting dialog messages only, as the formatting is not perfect.
    fn limit_width(self) -> String;
}
impl<T> LimitWidthExt<T> for T
where
    T: std::fmt::Display,
{
    fn limit_width(self) -> String {
        limit_width(self)
    }
}

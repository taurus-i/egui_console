use egui::Color32;
use crate::console::{TextStyle, StyledText};
use crate::ConsoleWindow;

/// Supported languages for syntax highlighting
pub enum Language {
    Rust,
    Python,
    JavaScript,
    HTML,
    CSS,
    JSON,
    Shell,
    Plaintext,
}

/// Basic syntax highlighter for displaying code in the terminal
pub struct SyntaxHighlighter {
    language: Language,
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter for the given language
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    /// Highlight a code snippet and return styled text segments
    pub fn highlight(&self, code: &str) -> Vec<StyledText> {
        match self.language {
            Language::Rust => self.highlight_rust(code),
            Language::Python => self.highlight_python(code),
            Language::JavaScript => self.highlight_javascript(code),
            Language::Shell => self.highlight_shell(code),
            Language::JSON => self.highlight_json(code),
            // For other languages, use a simple implementation for now
            _ => vec![StyledText::normal(code.to_string())]
        }
    }

    /// Highlight Rust code
    fn highlight_rust(&self, code: &str) -> Vec<StyledText> {
        let mut segments = Vec::new();

        // Keywords for Rust
        let keywords = [
            "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "use",
            "mod", "self", "Self", "for", "if", "else", "while", "loop", "break",
            "continue", "return", "match", "as", "in", "where", "unsafe", "extern",
            "static", "const", "async", "await", "dyn", "move", "ref"
        ];

        // Parse the code and add segments with different styles
        let lines = code.lines();
        for line in lines {
            // Handle comments
            if let Some(comment_idx) = line.find("//") {
                if comment_idx > 0 {
                    segments.push(self.process_code_line(&line[..comment_idx], &keywords));
                }
                segments.push(StyledText::new(
                    format!("{}{}", &line[comment_idx..], "\n"),
                    TextStyle::Custom(Color32::from_rgb(106, 153, 85))
                ));
                continue;
            }

            segments.push(self.process_code_line(line, &keywords));
            segments.push(StyledText::normal("\n".to_string()));
        }

        segments
    }

    fn process_code_line(&self, line: &str, keywords: &[&str]) -> StyledText {
        // This is a very simplified implementation
        // A real syntax highlighter would use a more sophisticated parsing approach

        for keyword in keywords {
            if line.contains(keyword) {
                // Simple case: line contains a keyword
                let highlighted = line.replace(
                    keyword, 
                    &format!("\u{001b}[1;35m{}\u{001b}[0m", keyword)
                );
                return StyledText::custom(
                    highlighted,
                    Color32::from_rgb(220, 220, 220)
                );
            }
        }

        // String literals
        if line.contains('"') {
            // Very simplified string highlighting - a real implementation would be more robust
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    let string_literal = &line[start..=start + end + 1];
                    let result = line.replace(
                        string_literal,
                        &format!("\u{001b}[0;32m{}\u{001b}[0m", string_literal)
                    );
                    return StyledText::custom(
                        result,
                        Color32::from_rgb(220, 220, 220)
                    );
                }
            }
        }

        // Default style for code
        StyledText::normal(line.to_string())
    }

    /// Highlight Python code
    fn highlight_python(&self, code: &str) -> Vec<StyledText> {
        let mut segments = Vec::new();

        // Keywords for Python
        let keywords = [
            "def", "class", "if", "else", "elif", "for", "while", "try", "except",
            "finally", "with", "as", "import", "from", "return", "yield", "break",
            "continue", "pass", "raise", "in", "is", "not", "and", "or", "lambda",
            "None", "True", "False", "global", "nonlocal", "del", "assert"
        ];

        // Similar approach as Rust but with Python-specific handling
        let lines = code.lines();
        for line in lines {
            // Handle comments
            if let Some(comment_idx) = line.find("#") {
                if comment_idx > 0 {
                    segments.push(self.process_code_line(&line[..comment_idx], &keywords));
                }
                segments.push(StyledText::new(
                    format!("{}{}", &line[comment_idx..], "\n"),
                    TextStyle::Custom(Color32::from_rgb(106, 153, 85))
                ));
                continue;
            }

            segments.push(self.process_code_line(line, &keywords));
            segments.push(StyledText::normal("\n".to_string()));
        }

        segments
    }

    /// Highlight JavaScript code
    fn highlight_javascript(&self, code: &str) -> Vec<StyledText> {
        let mut segments = Vec::new();

        // Keywords for JavaScript
        let keywords = [
            "function", "const", "let", "var", "if", "else", "for", "while", "do",
            "switch", "case", "default", "break", "continue", "return", "try", "catch",
            "finally", "throw", "class", "new", "this", "super", "extends", "import",
            "export", "from", "as", "async", "await", "yield", "typeof", "instanceof",
            "in", "of", "null", "undefined", "true", "false"
        ];

        // Similar implementation to Rust but with JavaScript-specific handling
        let lines = code.lines();
        for line in lines {
            // Handle comments
            if let Some(comment_idx) = line.find("//") {
                if comment_idx > 0 {
                    segments.push(self.process_code_line(&line[..comment_idx], &keywords));
                }
                segments.push(StyledText::new(
                    format!("{}{}", &line[comment_idx..], "\n"),
                    TextStyle::Custom(Color32::from_rgb(106, 153, 85))
                ));
                continue;
            }

            segments.push(self.process_code_line(line, &keywords));
            segments.push(StyledText::normal("\n".to_string()));
        }

        segments
    }

    /// Highlight shell code
    fn highlight_shell(&self, code: &str) -> Vec<StyledText> {
        let mut segments = Vec::new();

        // Keywords for shell
        let keywords = [
            "if", "then", "else", "elif", "fi", "for", "do", "done", "while", "until",
            "case", "esac", "function", "select", "time", "cd", "echo", "exit", "export",
            "pwd", "read", "source", "sudo", "alias", "unalias", "set", "unset"
        ];

        // Similar approach as others but with shell-specific handling
        let lines = code.lines();
        for line in lines {
            // Handle comments
            if let Some(comment_idx) = line.find("#") {
                if comment_idx > 0 {
                    segments.push(self.process_code_line(&line[..comment_idx], &keywords));
                }
                segments.push(StyledText::new(
                    format!("{}{}", &line[comment_idx..], "\n"),
                    TextStyle::Custom(Color32::from_rgb(106, 153, 85))
                ));
                continue;
            }

            segments.push(self.process_code_line(line, &keywords));
            segments.push(StyledText::normal("\n".to_string()));
        }

        segments
    }

    /// Highlight JSON code
    fn highlight_json(&self, code: &str) -> Vec<StyledText> {
        let mut segments = Vec::new();

        // Very simplistic JSON highlighting
        let lines = code.lines();
        for line in lines {
            // Highlight keys
            if line.contains(":") {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();

                    // Key with quotes
                    if key.starts_with('"') && key.ends_with('"') {
                        segments.push(StyledText::new(
                            format!("{}: ", key),
                            TextStyle::Custom(Color32::from_rgb(156, 220, 254))
                        ));
                    } else {
                        segments.push(StyledText::normal(format!("{}: ", key)));
                    }

                    // Value handling
                    if value.starts_with('"') && value.ends_with('"') {
                        // String value
                        segments.push(StyledText::new(
                            format!("{}{}", value, if value.ends_with(',') { "" } else { "," }),
                            TextStyle::Custom(Color32::from_rgb(206, 145, 120))
                        ));
                    } else if value == "true" || value == "false" || value == "null" ||
                              (value.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-')) {
                        // Boolean, null, or number
                        segments.push(StyledText::new(
                            format!("{}{}", value, if value.ends_with(',') { "" } else { "," }),
                            TextStyle::Custom(Color32::from_rgb(181, 206, 168))
                        ));
                    } else {
                        segments.push(StyledText::normal(format!("{}\n", value)));
                    }
                } else {
                    segments.push(StyledText::normal(format!("{}\n", line)));
                }
            } else {
                // Brackets and braces
                if line.trim() == "{" || line.trim() == "}" || 
                   line.trim() == "[" || line.trim() == "]" {
                    segments.push(StyledText::new(
                        format!("{}\n", line),
                        TextStyle::Custom(Color32::from_rgb(220, 220, 220))
                    ));
                } else {
                    segments.push(StyledText::normal(format!("{}\n", line)));
                }
            }
        }

        segments
    }
}

impl ConsoleWindow {
    /// Write code with syntax highlighting
    pub fn write_code(&mut self, code: &str, language: Language) {
        let highlighter = SyntaxHighlighter::new(language);
        let highlighted = highlighter.highlight(code);

        for segment in highlighted {
            self.write_styled(segment);
        }
    }
}

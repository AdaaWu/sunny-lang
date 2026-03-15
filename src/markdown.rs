/// Sunny 內建 Markdown → HTML 渲染器
///
/// 支援：
/// - # ~ ###### 標題
/// - **粗體**, *斜體*, `行內代碼`
/// - ``` 代碼區塊
/// - - / * 無序列表
/// - > 引用
/// - --- 分隔線
/// - [連結](url)
/// - 空行產生段落分隔

pub fn render(input: &str) -> String {
    let mut html = String::new();
    let lines: Vec<&str> = input.lines().collect();
    let len = lines.len();
    let mut i = 0;

    while i < len {
        let line = lines[i];

        // 代碼區塊
        if line.starts_with("```") {
            let lang = line[3..].trim();
            i += 1;
            let mut code = String::new();
            while i < len && !lines[i].starts_with("```") {
                if !code.is_empty() {
                    code.push('\n');
                }
                code.push_str(lines[i]);
                i += 1;
            }
            if i < len {
                i += 1; // skip closing ```
            }
            let escaped = escape_html(&code);
            if lang.is_empty() {
                html.push_str(&format!("<pre><code>{}</code></pre>\n", escaped));
            } else {
                html.push_str(&format!(
                    "<pre><code class=\"language-{}\">{}</code></pre>\n",
                    lang, escaped
                ));
            }
            continue;
        }

        // 分隔線
        let trimmed = line.trim();
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            html.push_str("<hr>\n");
            i += 1;
            continue;
        }

        // 標題
        if line.starts_with('#') {
            let level = line.chars().take_while(|c| *c == '#').count().min(6);
            let text = line[level..].trim();
            let inline = render_inline(text);
            html.push_str(&format!("<h{}>{}</h{}>\n", level, inline, level));
            i += 1;
            continue;
        }

        // 引用
        if line.starts_with("> ") || line == ">" {
            let mut quote_lines = Vec::new();
            while i < len && (lines[i].starts_with("> ") || lines[i] == ">") {
                let content = if lines[i] == ">" {
                    ""
                } else {
                    &lines[i][2..]
                };
                quote_lines.push(content);
                i += 1;
            }
            let inner = quote_lines.join("\n");
            let inner_html = render(&inner);
            html.push_str(&format!("<blockquote>{}</blockquote>\n", inner_html.trim()));
            continue;
        }

        // 無序列表
        if line.starts_with("- ") || line.starts_with("* ") {
            html.push_str("<ul>\n");
            while i < len && (lines[i].starts_with("- ") || lines[i].starts_with("* ")) {
                let item = &lines[i][2..];
                let inline = render_inline(item);
                html.push_str(&format!("<li>{}</li>\n", inline));
                i += 1;
            }
            html.push_str("</ul>\n");
            continue;
        }

        // 空行
        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        // 段落：收集連續非空行
        let mut para_lines = Vec::new();
        while i < len {
            let l = lines[i].trim();
            if l.is_empty()
                || l.starts_with('#')
                || l.starts_with("```")
                || l.starts_with("> ")
                || l.starts_with("- ")
                || l.starts_with("* ")
                || l == "---"
                || l == "***"
            {
                break;
            }
            para_lines.push(lines[i]);
            i += 1;
        }
        let text = para_lines.join(" ");
        let inline = render_inline(&text);
        html.push_str(&format!("<p>{}</p>\n", inline));
    }

    html
}

/// 行內格式：**粗體**, *斜體*, `code`, [link](url)
fn render_inline(input: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // **粗體**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_closing(&chars, i + 2, "**") {
                let inner: String = chars[i + 2..end].iter().collect();
                result.push_str(&format!("<strong>{}</strong>", escape_html(&inner)));
                i = end + 2;
                continue;
            }
        }

        // *斜體*
        if chars[i] == '*' && (i + 1 >= len || chars[i + 1] != '*') {
            if let Some(end) = find_closing_char(&chars, i + 1, '*') {
                let inner: String = chars[i + 1..end].iter().collect();
                result.push_str(&format!("<em>{}</em>", escape_html(&inner)));
                i = end + 1;
                continue;
            }
        }

        // `行內代碼`
        if chars[i] == '`' {
            if let Some(end) = find_closing_char(&chars, i + 1, '`') {
                let inner: String = chars[i + 1..end].iter().collect();
                result.push_str(&format!("<code>{}</code>", escape_html(&inner)));
                i = end + 1;
                continue;
            }
        }

        // [連結](url)
        if chars[i] == '[' {
            if let Some(text_end) = find_closing_char(&chars, i + 1, ']') {
                if text_end + 1 < len && chars[text_end + 1] == '(' {
                    if let Some(url_end) = find_closing_char(&chars, text_end + 2, ')') {
                        let text: String = chars[i + 1..text_end].iter().collect();
                        let url: String = chars[text_end + 2..url_end].iter().collect();
                        result.push_str(&format!(
                            "<a href=\"{}\">{}</a>",
                            escape_html(&url),
                            escape_html(&text)
                        ));
                        i = url_end + 1;
                        continue;
                    }
                }
            }
        }

        // 普通字元需要 escape
        match chars[i] {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            c => result.push(c),
        }
        i += 1;
    }

    result
}

fn find_closing(chars: &[char], start: usize, pattern: &str) -> Option<usize> {
    let pat: Vec<char> = pattern.chars().collect();
    let pat_len = pat.len();
    for i in start..=chars.len().saturating_sub(pat_len) {
        if chars[i..i + pat_len] == pat[..] {
            return Some(i);
        }
    }
    None
}

fn find_closing_char(chars: &[char], start: usize, ch: char) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == ch {
            return Some(i);
        }
    }
    None
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading() {
        assert_eq!(render("# Hello"), "<h1>Hello</h1>\n");
        assert_eq!(render("## Sub"), "<h2>Sub</h2>\n");
        assert_eq!(render("### Deep"), "<h3>Deep</h3>\n");
    }

    #[test]
    fn test_paragraph() {
        assert_eq!(render("Hello world"), "<p>Hello world</p>\n");
    }

    #[test]
    fn test_bold_italic() {
        assert_eq!(render("**bold**"), "<p><strong>bold</strong></p>\n");
        assert_eq!(render("*italic*"), "<p><em>italic</em></p>\n");
    }

    #[test]
    fn test_inline_code() {
        assert_eq!(render("`code`"), "<p><code>code</code></p>\n");
    }

    #[test]
    fn test_link() {
        assert_eq!(
            render("[Sunny](https://sunny.dev)"),
            "<p><a href=\"https://sunny.dev\">Sunny</a></p>\n"
        );
    }

    #[test]
    fn test_code_block() {
        let input = "```sunny\nlit x = 1\n```";
        assert_eq!(
            render(input),
            "<pre><code class=\"language-sunny\">lit x = 1</code></pre>\n"
        );
    }

    #[test]
    fn test_unordered_list() {
        let input = "- one\n- two\n- three";
        assert_eq!(
            render(input),
            "<ul>\n<li>one</li>\n<li>two</li>\n<li>three</li>\n</ul>\n"
        );
    }

    #[test]
    fn test_blockquote() {
        assert_eq!(
            render("> quote"),
            "<blockquote><p>quote</p></blockquote>\n"
        );
    }

    #[test]
    fn test_hr() {
        assert_eq!(render("---"), "<hr>\n");
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(
            render("<script>alert(1)</script>"),
            "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>\n"
        );
    }

    #[test]
    fn test_full_document() {
        let input = r#"# Sunny Lang

A modern programming language.

## Features

- **Safe**: No null, no exceptions
- *Fast*: Powered by Rust
- `Simple`: Clean syntax

```sunny
fn helloShow() -> String {
    out "Hello!"
}
```

---

[Learn more](https://sunny.dev)"#;

        let html = render(input);
        assert!(html.contains("<h1>Sunny Lang</h1>"));
        assert!(html.contains("<h2>Features</h2>"));
        assert!(html.contains("<strong>Safe</strong>"));
        assert!(html.contains("<em>Fast</em>"));
        assert!(html.contains("<code>Simple</code>"));
        assert!(html.contains("<pre><code class=\"language-sunny\">"));
        assert!(html.contains("<hr>"));
        assert!(html.contains("<a href=\"https://sunny.dev\">"));
    }
}

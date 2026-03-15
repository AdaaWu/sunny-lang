use crate::environment::Value;

/// Sunny 模板引擎
///
/// 將模板中的 `{{ key }}` 替換為 vars map 中對應的值。
/// 支援：
/// - `{{ key }}` — 簡單變數替換
/// - `{{ key }}` 中的空白會被忽略（`{{key}}` 和 `{{ key }}` 皆可）
///
/// 範例：
/// ```ignore
/// render("<h1>{{ title }}</h1>", &[("title".to_string(), Value::Str("Hello".to_string()))])
/// // => "<h1>Hello</h1>"
/// ```

pub fn render(template: &str, vars: &[(String, Value)]) -> String {
    let mut result = String::new();
    let chars: Vec<char> = template.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // 偵測 {{
        if i + 1 < len && chars[i] == '{' && chars[i + 1] == '{' {
            i += 2;
            // 跳過空白
            while i < len && chars[i] == ' ' {
                i += 1;
            }
            // 讀取 key
            let key_start = i;
            while i < len && chars[i] != '}' && chars[i] != ' ' {
                i += 1;
            }
            let key: String = chars[key_start..i].iter().collect();
            // 跳過尾部空白
            while i < len && chars[i] == ' ' {
                i += 1;
            }
            // 跳過 }}
            if i + 1 < len && chars[i] == '}' && chars[i + 1] == '}' {
                i += 2;
            }

            // 查找 key 對應的值
            let value = vars
                .iter()
                .find(|(k, _)| k == &key)
                .map(|(_, v)| format_value(v))
                .unwrap_or_default();
            result.push_str(&value);
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

fn format_value(val: &Value) -> String {
    match val {
        Value::Str(s) => s.clone(),
        other => format!("{}", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_replacement() {
        let vars = vec![("name".to_string(), Value::Str("Sunny".to_string()))];
        assert_eq!(render("Hello {{ name }}!", &vars), "Hello Sunny!");
    }

    #[test]
    fn test_no_spaces() {
        let vars = vec![("x".to_string(), Value::Int(42))];
        assert_eq!(render("value={{x}}", &vars), "value=42");
    }

    #[test]
    fn test_multiple_vars() {
        let vars = vec![
            ("title".to_string(), Value::Str("Sunny Lang".to_string())),
            ("version".to_string(), Value::Int(1)),
        ];
        let template = "<h1>{{ title }}</h1><p>v{{ version }}</p>";
        assert_eq!(
            render(template, &vars),
            "<h1>Sunny Lang</h1><p>v1</p>"
        );
    }

    #[test]
    fn test_missing_var_empty() {
        let vars: Vec<(String, Value)> = vec![];
        assert_eq!(render("{{ missing }}", &vars), "");
    }

    #[test]
    fn test_no_template_markers() {
        let vars = vec![("x".to_string(), Value::Int(1))];
        assert_eq!(render("no templates here", &vars), "no templates here");
    }

    #[test]
    fn test_full_html_template() {
        let vars = vec![
            ("title".to_string(), Value::Str("Sunny Docs".to_string())),
            (
                "content".to_string(),
                Value::Str("<p>Hello</p>".to_string()),
            ),
        ];
        let template = r#"<!DOCTYPE html>
<html>
<head><title>{{ title }}</title></head>
<body>{{ content }}</body>
</html>"#;
        let result = render(template, &vars);
        assert!(result.contains("<title>Sunny Docs</title>"));
        assert!(result.contains("<body><p>Hello</p></body>"));
    }
}

use crate::ast::*;

/// Resource Action 合法後綴
const VALID_SUFFIXES: &[&str] = &["index", "show", "store", "update", "remove"];

/// Linter 檢查結果
#[derive(Debug, Clone, PartialEq)]
pub struct LintWarning {
    pub function_name: String,
    pub message: String,
}

/// Sunny 靜態檢查器
/// 在執行前掃描 AST，強制規範檢查
pub fn lint(program: &Program) -> Vec<LintWarning> {
    let mut warnings = Vec::new();
    for stmt in &program.statements {
        lint_statement(stmt, &mut warnings);
    }
    warnings
}

fn lint_statement(stmt: &Statement, warnings: &mut Vec<LintWarning>) {
    match stmt {
        Statement::Expression(expr) => lint_expression(expr, warnings),
        Statement::Lit { value, .. } => lint_expression(value, warnings),
        Statement::Glow { value, .. } => lint_expression(value, warnings),
        Statement::Assign { value, .. } => lint_expression(value, warnings),
        Statement::Out(expr) => lint_expression(expr, warnings),
        Statement::Import { .. } => {}
    }
}

fn lint_expression(expr: &Expression, warnings: &mut Vec<LintWarning>) {
    match expr {
        Expression::Function { name, body, .. } => {
            // 匿名函數（閉包）不需要檢查命名規範
            if !name.is_empty() {
                // SCREAMING_SNAKE_CASE 常數函數跳過 Resource Action 檢查
                if is_screaming_snake_case(name) {
                    // 常數命名，合法
                } else {
                    // 檢查 Resource Action 後綴
                    check_function_suffix(name, warnings);

                    // 檢查 camelCase（首字母小寫）
                    if let Some(first) = name.chars().next() {
                        if first.is_uppercase() {
                            warnings.push(LintWarning {
                                function_name: name.clone(),
                                message: format!(
                                    "function '{}' must use camelCase (start with lowercase)",
                                    name
                                ),
                            });
                        }
                    }
                }
            }

            // 遞迴檢查函數體內的巢狀函數
            for stmt in body {
                lint_statement(stmt, warnings);
            }
        }

        // 遞迴走訪其他含有子表達式的節點
        Expression::If {
            condition,
            consequence,
            alternative,
        } => {
            lint_expression(condition, warnings);
            for stmt in consequence {
                lint_statement(stmt, warnings);
            }
            if let Some(alt) = alternative {
                for stmt in alt {
                    lint_statement(stmt, warnings);
                }
            }
        }
        Expression::Match { subject, arms } => {
            lint_expression(subject, warnings);
            for arm in arms {
                for stmt in &arm.body {
                    lint_statement(stmt, warnings);
                }
            }
        }
        Expression::For { body, .. } => {
            for stmt in body {
                lint_statement(stmt, warnings);
            }
        }
        Expression::Ray { body } => {
            for stmt in body {
                lint_statement(stmt, warnings);
            }
        }
        Expression::While { condition, body } => {
            lint_expression(condition, warnings);
            for stmt in body {
                lint_statement(stmt, warnings);
            }
        }
        Expression::Range { .. } => {}
        _ => {}
    }
}

/// 判斷是否為 SCREAMING_SNAKE_CASE（全大寫 + 底線）
fn is_screaming_snake_case(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && name.chars().any(|c| c.is_ascii_uppercase())
}

fn check_function_suffix(name: &str, warnings: &mut Vec<LintWarning>) {
    let name_lower = name.to_lowercase();
    for suffix in VALID_SUFFIXES {
        if name_lower.ends_with(suffix) {
            return;
        }
    }
    warnings.push(LintWarning {
        function_name: name.to_string(),
        message: format!(
            "function '{}' must end with a Resource Action suffix: index, show, store, update, remove",
            name
        ),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn lint_code(input: &str) -> Vec<LintWarning> {
        let mut parser = Parser::new(input);
        let program = parser.parse();
        assert!(parser.errors.is_empty(), "parse errors: {:?}", parser.errors);
        lint(&program)
    }

    // ── 合法命名 ──

    #[test]
    fn test_valid_suffix_index() {
        let warnings = lint_code("fn userIndex() -> String { out \"ok\" }");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_valid_suffix_show() {
        let warnings = lint_code("fn bookShow(id: Int) -> String { out \"ok\" }");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_valid_suffix_store() {
        let warnings = lint_code("fn orderStore(data: String) -> String { out \"ok\" }");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_valid_suffix_update() {
        let warnings = lint_code("fn statusUpdate(id: Int) -> String { out \"ok\" }");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_valid_suffix_remove() {
        let warnings = lint_code("fn fileRemove(id: Int) -> String { out \"ok\" }");
        assert!(warnings.is_empty());
    }

    // ── 違規命名 ──

    #[test]
    fn test_missing_suffix() {
        let warnings = lint_code("fn getData() -> String { out \"ok\" }");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("Resource Action suffix"));
        assert_eq!(warnings[0].function_name, "getData");
    }

    #[test]
    fn test_uppercase_start() {
        let warnings = lint_code("fn UserIndex() -> String { out \"ok\" }");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("camelCase"));
    }

    #[test]
    fn test_both_violations() {
        let warnings = lint_code("fn GetData() -> String { out \"ok\" }");
        assert_eq!(warnings.len(), 2); // 大寫開頭 + 缺少後綴
    }

    // ── 多函數 ──

    #[test]
    fn test_multiple_functions() {
        let input = r#"
fn userIndex() -> String { out "ok" }
fn getData() -> String { out "bad" }
fn bookShow(id: Int) -> String { out "ok" }
"#;
        let warnings = lint_code(input);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].function_name, "getData");
    }

    // ── 巢狀函數也檢查 ──

    #[test]
    fn test_nested_function_checked() {
        let input = r#"
fn outerIndex() -> String {
    fn helper() -> String { out "inner" }
    out "ok"
}
"#;
        let warnings = lint_code(input);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].function_name, "helper");
    }

    // ── 整合測試：SPEC.md 範例全部合法 ──

    #[test]
    fn test_spec_examples_pass() {
        let input = r#"
fn dataShow(id: Int) -> String | Shadow {
    if id < 0 {
        out Shadow("ID must be positive")
    }
    out "Valid Data"
}
"#;
        let warnings = lint_code(input);
        assert!(warnings.is_empty());
    }

    // ── SCREAMING_SNAKE_CASE 常數 ──

    #[test]
    fn test_screaming_snake_case_allowed() {
        let warnings = lint_code("fn MAX_RETRY() -> Int { out 3 }");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_screaming_snake_case_with_digits() {
        let warnings = lint_code("fn HTTP_404() -> String { out \"not found\" }");
        assert!(warnings.is_empty());
    }

    // ── 匿名函數不檢查 ──

    #[test]
    fn test_anonymous_function_no_lint() {
        let warnings = lint_code("lit add = fn(x, y) { out x + y }");
        assert!(warnings.is_empty());
    }
}

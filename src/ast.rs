/// Sunny 語言抽象語法樹 (AST) 定義
/// 所有原始碼經過 Parser 後都會轉換成這些節點

/// 程式 = 一系列語句
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// 語句節點
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// `lit x = 10` — 不可變綁定
    Lit {
        name: String,
        value: Expression,
    },

    /// `glow x = 10` — 可變綁定
    Glow {
        name: String,
        value: Expression,
    },

    /// `x = 20` — 重新賦值（僅限 glow 變數）
    Assign {
        name: String,
        value: Expression,
    },

    /// `out value` — 回傳值
    Out(Expression),

    /// `import "path/to/file.sunny"` — 匯入模組
    Import {
        path: String,
    },

    /// 單獨的表達式語句（如函數呼叫）
    Expression(Expression),
}

/// 表達式節點
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// 整數字面量: `42`
    IntLiteral(i64),

    /// 浮點數字面量: `3.14`
    FloatLiteral(f64),

    /// 字串字面量: `"hello"`
    StringLiteral(String),

    /// 布林字面量: `true`, `false`
    BoolLiteral(bool),

    /// 識別符: `userName`
    Ident(String),

    /// 前綴表達式: `-x`, `!flag`, `not flag`
    Prefix {
        operator: PrefixOp,
        right: Box<Expression>,
    },

    /// 中綴表達式: `a + b`, `x == y`, `p and q`
    Infix {
        left: Box<Expression>,
        operator: InfixOp,
        right: Box<Expression>,
    },

    /// 函數定義: `fn bookShow(id: Int) -> String | Shadow { ... }`
    Function {
        name: String,
        params: Vec<Param>,
        return_type: Option<TypeAnnotation>,
        body: Vec<Statement>,
    },

    /// 函數呼叫: `print("hello")`, `dataShow(1)`
    Call {
        function: Box<Expression>,
        args: Vec<Expression>,
    },

    /// if 條件式: `if x > 0 { ... } else { ... }`
    If {
        condition: Box<Expression>,
        consequence: Vec<Statement>,
        alternative: Option<Vec<Statement>>,
    },

    /// match 模式匹配:
    /// ```sunny
    /// match result {
    ///     is String val -> print(val)
    ///     is Shadow s -> print(s.message)
    /// }
    /// ```
    Match {
        subject: Box<Expression>,
        arms: Vec<MatchArm>,
    },

    /// for 迴圈: `for item in list { ... }`
    For {
        item: String,
        iterable: Box<Expression>,
        body: Vec<Statement>,
    },

    /// List 字面量: `[1, 2, 3]`
    ListLiteral(Vec<Expression>),

    /// Map 字面量: `{"id": 1, "name": "Sunny"}`
    MapLiteral(Vec<(Expression, Expression)>),

    /// 索引存取: `list[0]`, `map["key"]`
    Index {
        left: Box<Expression>,
        index: Box<Expression>,
    },

    /// 屬性存取: `shadow.message`
    Dot {
        left: Box<Expression>,
        field: String,
    },

    /// ray 併發: `ray { ... }`
    Ray {
        body: Vec<Statement>,
    },

    /// Shadow 建構: `Shadow("error message")`
    ShadowLiteral(Box<Expression>),

    /// while 迴圈: `while condition { ... }`
    While {
        condition: Box<Expression>,
        body: Vec<Statement>,
    },

    /// Range: `0..10`
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
    },
}

/// 函數參數
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_ann: TypeAnnotation,
}

/// 型別標註
#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation {
    Int,
    Float,
    Str,
    Bool,
    List,
    Map,
    Shadow,
    /// 聯合型別: `String | Shadow`
    Union(Vec<TypeAnnotation>),
}

/// match 的單個分支: `is String val -> expr`
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub type_ann: TypeAnnotation,
    pub binding: String,
    pub body: Vec<Statement>,
}

/// 前綴運算子
#[derive(Debug, Clone, PartialEq)]
pub enum PrefixOp {
    Negate, // -
    Bang,   // !
    Not,    // not
}

/// 中綴運算子
#[derive(Debug, Clone, PartialEq)]
pub enum InfixOp {
    // 算術
    Add,     // +
    Sub,     // -
    Mul,     // *
    Div,     // /
    Mod,     // %

    // 比較
    Eq,      // ==
    NotEq,   // !=
    Lt,      // <
    Gt,      // >
    LtEq,    // <=
    GtEq,    // >=

    // 邏輯
    And,     // and, &&
    Or,      // or, ||
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lit_statement() {
        let program = Program {
            statements: vec![Statement::Lit {
                name: "x".to_string(),
                value: Expression::IntLiteral(42),
            }],
        };
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Lit { name, value } => {
                assert_eq!(name, "x");
                assert_eq!(*value, Expression::IntLiteral(42));
            }
            _ => panic!("expected Lit statement"),
        }
    }

    #[test]
    fn test_function_expression() {
        // fn bookShow(id: Int) -> String { output "hello" }
        let func = Expression::Function {
            name: "bookShow".to_string(),
            params: vec![Param {
                name: "id".to_string(),
                type_ann: TypeAnnotation::Int,
            }],
            return_type: Some(TypeAnnotation::Str),
            body: vec![Statement::Out(Expression::StringLiteral(
                "hello".to_string(),
            ))],
        };
        match &func {
            Expression::Function { name, params, .. } => {
                assert_eq!(name, "bookShow");
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "id");
                assert_eq!(params[0].type_ann, TypeAnnotation::Int);
            }
            _ => panic!("expected Function expression"),
        }
    }

    #[test]
    fn test_union_type() {
        let t = TypeAnnotation::Union(vec![TypeAnnotation::Str, TypeAnnotation::Shadow]);
        match &t {
            TypeAnnotation::Union(types) => {
                assert_eq!(types.len(), 2);
                assert_eq!(types[0], TypeAnnotation::Str);
                assert_eq!(types[1], TypeAnnotation::Shadow);
            }
            _ => panic!("expected Union type"),
        }
    }

    #[test]
    fn test_infix_expression() {
        // a + b
        let expr = Expression::Infix {
            left: Box::new(Expression::Ident("a".to_string())),
            operator: InfixOp::Add,
            right: Box::new(Expression::Ident("b".to_string())),
        };
        match &expr {
            Expression::Infix { operator, .. } => {
                assert_eq!(*operator, InfixOp::Add);
            }
            _ => panic!("expected Infix expression"),
        }
    }

    #[test]
    fn test_if_expression() {
        // if x > 0 { out "positive" } else { out "non-positive" }
        let expr = Expression::If {
            condition: Box::new(Expression::Infix {
                left: Box::new(Expression::Ident("x".to_string())),
                operator: InfixOp::Gt,
                right: Box::new(Expression::IntLiteral(0)),
            }),
            consequence: vec![Statement::Out(Expression::StringLiteral(
                "positive".to_string(),
            ))],
            alternative: Some(vec![Statement::Out(Expression::StringLiteral(
                "non-positive".to_string(),
            ))]),
        };
        match &expr {
            Expression::If { alternative, .. } => {
                assert!(alternative.is_some());
            }
            _ => panic!("expected If expression"),
        }
    }

    #[test]
    fn test_match_expression() {
        let expr = Expression::Match {
            subject: Box::new(Expression::Ident("result".to_string())),
            arms: vec![
                MatchArm {
                    type_ann: TypeAnnotation::Str,
                    binding: "val".to_string(),
                    body: vec![Statement::Expression(Expression::Call {
                        function: Box::new(Expression::Ident("print".to_string())),
                        args: vec![Expression::Ident("val".to_string())],
                    })],
                },
                MatchArm {
                    type_ann: TypeAnnotation::Shadow,
                    binding: "s".to_string(),
                    body: vec![Statement::Expression(Expression::Call {
                        function: Box::new(Expression::Ident("print".to_string())),
                        args: vec![Expression::Dot {
                            left: Box::new(Expression::Ident("s".to_string())),
                            field: "message".to_string(),
                        }],
                    })],
                },
            ],
        };
        match &expr {
            Expression::Match { arms, .. } => {
                assert_eq!(arms.len(), 2);
                assert_eq!(arms[0].binding, "val");
                assert_eq!(arms[1].type_ann, TypeAnnotation::Shadow);
            }
            _ => panic!("expected Match expression"),
        }
    }

    #[test]
    fn test_for_expression() {
        // for item in items { ... }
        let expr = Expression::For {
            item: "item".to_string(),
            iterable: Box::new(Expression::Ident("items".to_string())),
            body: vec![],
        };
        match &expr {
            Expression::For { item, .. } => {
                assert_eq!(item, "item");
            }
            _ => panic!("expected For expression"),
        }
    }

    #[test]
    fn test_list_and_map_literals() {
        let list = Expression::ListLiteral(vec![
            Expression::IntLiteral(1),
            Expression::IntLiteral(2),
        ]);
        match &list {
            Expression::ListLiteral(items) => assert_eq!(items.len(), 2),
            _ => panic!("expected ListLiteral"),
        }

        let map = Expression::MapLiteral(vec![(
            Expression::StringLiteral("id".to_string()),
            Expression::IntLiteral(1),
        )]);
        match &map {
            Expression::MapLiteral(pairs) => assert_eq!(pairs.len(), 1),
            _ => panic!("expected MapLiteral"),
        }
    }
}

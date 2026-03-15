use crate::ast::*;
use crate::lexer::Lexer;
use crate::token::Token;

/// 運算子優先級（數值越大越優先）
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Precedence {
    Lowest = 0,
    Or = 1,        // or, ||
    And = 2,       // and, &&
    Equals = 3,    // ==, !=
    LessGt = 4,    // <, >, <=, >=
    Sum = 5,       // +, -
    Product = 6,   // *, /, %
    Prefix = 7,    // -x, !x, not x
    Call = 8,      // fn()
    Index = 9,     // a[0], a.b, 0..10
}

fn token_precedence(tok: &Token) -> Precedence {
    match tok {
        Token::Or | Token::PipePipe => Precedence::Or,
        Token::And | Token::AmpAmp => Precedence::And,
        Token::Equal | Token::NotEqual => Precedence::Equals,
        Token::LessThan | Token::GreaterThan | Token::LessEqual | Token::GreaterEqual => {
            Precedence::LessGt
        }
        Token::Plus | Token::Minus => Precedence::Sum,
        Token::Star | Token::Slash | Token::Percent => Precedence::Product,
        Token::DotDot => Precedence::Sum, // range binds tighter than comparison but looser than +/-
        Token::LParen => Precedence::Call,
        Token::LBracket | Token::Dot => Precedence::Index,
        _ => Precedence::Lowest,
    }
}

/// Sunny 語言語法解析器
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub errors: Vec<String>,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        Parser {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    // ── 基礎工具 ──

    fn cur(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos + 1).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, expected: &Token) -> bool {
        if self.cur() == expected {
            self.advance();
            true
        } else {
            self.errors.push(format!(
                "expected {}, got {}",
                expected,
                self.cur()
            ));
            false
        }
    }

    fn expect_ident(&mut self) -> Option<String> {
        match self.cur().clone() {
            Token::Ident(name) => {
                self.advance();
                Some(name)
            }
            _ => {
                self.errors
                    .push(format!("expected identifier, got {}", self.cur()));
                None
            }
        }
    }

    // ── 程式解析入口 ──

    pub fn parse(&mut self) -> Program {
        let mut statements = Vec::new();
        while *self.cur() != Token::Eof {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                self.advance(); // 跳過錯誤 token，避免無窮迴圈
            }
        }
        Program { statements }
    }

    // ── 語句解析 ──

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur() {
            Token::Lit => self.parse_lit(),
            Token::Glow => self.parse_glow(),
            Token::Out => self.parse_out(),
            Token::Import => self.parse_import(),
            Token::Ident(_) if *self.peek() == Token::Assign => self.parse_assign(),
            _ => self.parse_expression_statement(),
        }
    }

    /// `lit name = expr`
    fn parse_lit(&mut self) -> Option<Statement> {
        self.advance(); // skip `lit`
        let name = self.expect_ident()?;
        self.expect(&Token::Assign);
        let value = self.parse_expression(Precedence::Lowest)?;
        Some(Statement::Lit { name, value })
    }

    /// `glow name = expr`
    fn parse_glow(&mut self) -> Option<Statement> {
        self.advance(); // skip `glow`
        let name = self.expect_ident()?;
        self.expect(&Token::Assign);
        let value = self.parse_expression(Precedence::Lowest)?;
        Some(Statement::Glow { name, value })
    }

    /// `name = expr`
    fn parse_assign(&mut self) -> Option<Statement> {
        let name = self.expect_ident()?;
        self.expect(&Token::Assign);
        let value = self.parse_expression(Precedence::Lowest)?;
        Some(Statement::Assign { name, value })
    }

    /// `import "path/to/file.sunny"`
    fn parse_import(&mut self) -> Option<Statement> {
        self.advance(); // skip `import`
        match self.cur().clone() {
            Token::StringLiteral(path) => {
                self.advance();
                Some(Statement::Import { path })
            }
            _ => {
                self.errors.push(format!("import expects a string path, got {}", self.cur()));
                None
            }
        }
    }

    /// `out expr`
    fn parse_out(&mut self) -> Option<Statement> {
        self.advance(); // skip `out`
        let value = self.parse_expression(Precedence::Lowest)?;
        Some(Statement::Out(value))
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expression(Precedence::Lowest)?;
        Some(Statement::Expression(expr))
    }

    // ── Pratt Parsing 核心 ──

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        // 前綴解析
        let mut left = self.parse_prefix()?;

        // 中綴解析：只要下一個運算子的優先級更高，就繼續組合
        while *self.cur() != Token::Eof && precedence < self.cur_precedence() {
            left = match self.cur() {
                Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Percent
                | Token::Equal
                | Token::NotEqual
                | Token::LessThan
                | Token::GreaterThan
                | Token::LessEqual
                | Token::GreaterEqual
                | Token::And
                | Token::Or
                | Token::AmpAmp
                | Token::PipePipe => self.parse_infix(left)?,
                Token::DotDot => self.parse_range(left)?,
                Token::LParen => self.parse_call(left)?,
                Token::LBracket => self.parse_index(left)?,
                Token::Dot => self.parse_dot(left)?,
                _ => break,
            };
        }

        Some(left)
    }

    fn cur_precedence(&self) -> Precedence {
        token_precedence(self.cur())
    }

    // ── 前綴解析 ──

    fn parse_prefix(&mut self) -> Option<Expression> {
        match self.cur().clone() {
            Token::Int(v) => {
                self.advance();
                Some(Expression::IntLiteral(v))
            }
            Token::Float(v) => {
                self.advance();
                Some(Expression::FloatLiteral(v))
            }
            Token::StringLiteral(v) => {
                self.advance();
                Some(Expression::StringLiteral(v))
            }
            Token::Bool(v) => {
                self.advance();
                Some(Expression::BoolLiteral(v))
            }
            Token::Ident(name) => {
                self.advance();
                Some(Expression::Ident(name))
            }
            Token::Minus => {
                self.advance();
                let right = self.parse_expression(Precedence::Prefix)?;
                Some(Expression::Prefix {
                    operator: PrefixOp::Negate,
                    right: Box::new(right),
                })
            }
            Token::Bang => {
                self.advance();
                let right = self.parse_expression(Precedence::Prefix)?;
                Some(Expression::Prefix {
                    operator: PrefixOp::Bang,
                    right: Box::new(right),
                })
            }
            Token::Not => {
                self.advance();
                let right = self.parse_expression(Precedence::Prefix)?;
                Some(Expression::Prefix {
                    operator: PrefixOp::Not,
                    right: Box::new(right),
                })
            }
            Token::LParen => self.parse_grouped(),
            Token::Fn => self.parse_function(),
            Token::If => self.parse_if(),
            Token::Match => self.parse_match(),
            Token::For => self.parse_for(),
            Token::While => self.parse_while(),
            Token::Ray => self.parse_ray(),
            Token::LBracket => self.parse_list(),
            Token::LBrace => self.parse_map(),
            Token::TypeShadow => self.parse_shadow_constructor(),
            _ => {
                self.errors
                    .push(format!("unexpected token: {}", self.cur()));
                None
            }
        }
    }

    // ── 中綴解析 ──

    fn parse_infix(&mut self, left: Expression) -> Option<Expression> {
        let operator = match self.cur() {
            Token::Plus => InfixOp::Add,
            Token::Minus => InfixOp::Sub,
            Token::Star => InfixOp::Mul,
            Token::Slash => InfixOp::Div,
            Token::Percent => InfixOp::Mod,
            Token::Equal => InfixOp::Eq,
            Token::NotEqual => InfixOp::NotEq,
            Token::LessThan => InfixOp::Lt,
            Token::GreaterThan => InfixOp::Gt,
            Token::LessEqual => InfixOp::LtEq,
            Token::GreaterEqual => InfixOp::GtEq,
            Token::And | Token::AmpAmp => InfixOp::And,
            Token::Or | Token::PipePipe => InfixOp::Or,
            _ => {
                self.errors
                    .push(format!("unexpected infix operator: {}", self.cur()));
                return None;
            }
        };
        let prec = self.cur_precedence();
        self.advance(); // skip operator
        let right = self.parse_expression(prec)?;
        Some(Expression::Infix {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    // ── 複合表達式解析 ──

    /// `(expr)`
    fn parse_grouped(&mut self) -> Option<Expression> {
        self.advance(); // skip (
        let expr = self.parse_expression(Precedence::Lowest)?;
        self.expect(&Token::RParen);
        Some(expr)
    }

    /// `fn name(params) -> ReturnType { body }` 或匿名 `fn(params) { body }`
    fn parse_function(&mut self) -> Option<Expression> {
        self.advance(); // skip `fn`
        // 匿名函數: fn( 直接接參數
        let name = if *self.cur() == Token::LParen {
            String::new()
        } else {
            self.expect_ident()?
        };

        // 參數列表（匿名函數允許省略型別標註）
        self.expect(&Token::LParen);
        let params = if name.is_empty() {
            self.parse_closure_params()
        } else {
            self.parse_params()
        };
        self.expect(&Token::RParen);

        // 可選的回傳型別
        let return_type = if *self.cur() == Token::Arrow {
            self.advance();
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        // 函數體
        let body = self.parse_block()?;

        Some(Expression::Function {
            name,
            params,
            return_type,
            body,
        })
    }

    /// 閉包參數：只需要名稱，型別可選（預設 Int 作為佔位）
    fn parse_closure_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        if *self.cur() == Token::RParen {
            return params;
        }
        loop {
            if let Some(name) = self.expect_ident() {
                // 可選型別標註: `x: Int` 或只有 `x`
                let type_ann = if *self.cur() == Token::Colon {
                    self.advance();
                    self.parse_single_type().unwrap_or(TypeAnnotation::Int)
                } else {
                    TypeAnnotation::Int // 佔位，閉包不強制型別
                };
                params.push(Param { name, type_ann });
            }
            if *self.cur() != Token::Comma {
                break;
            }
            self.advance();
        }
        params
    }

    fn parse_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        if *self.cur() == Token::RParen {
            return params;
        }

        loop {
            if let Some(name) = self.expect_ident() {
                self.expect(&Token::Colon);
                if let Some(type_ann) = self.parse_single_type() {
                    params.push(Param { name, type_ann });
                }
            }
            if *self.cur() != Token::Comma {
                break;
            }
            self.advance(); // skip comma
        }
        params
    }

    /// 解析型別標註（可能含聯合型別 `String | Shadow`）
    fn parse_type_annotation(&mut self) -> Option<TypeAnnotation> {
        let first = self.parse_single_type()?;
        if *self.cur() == Token::Pipe {
            let mut types = vec![first];
            while *self.cur() == Token::Pipe {
                self.advance(); // skip |
                types.push(self.parse_single_type()?);
            }
            Some(TypeAnnotation::Union(types))
        } else {
            Some(first)
        }
    }

    fn parse_single_type(&mut self) -> Option<TypeAnnotation> {
        let t = match self.cur() {
            Token::TypeInt => TypeAnnotation::Int,
            Token::TypeFloat => TypeAnnotation::Float,
            Token::TypeString => TypeAnnotation::Str,
            Token::TypeBool => TypeAnnotation::Bool,
            Token::TypeList => TypeAnnotation::List,
            Token::TypeMap => TypeAnnotation::Map,
            Token::TypeShadow => TypeAnnotation::Shadow,
            _ => {
                self.errors
                    .push(format!("expected type, got {}", self.cur()));
                return None;
            }
        };
        self.advance();
        Some(t)
    }

    /// 解析 `{ statements }`
    fn parse_block(&mut self) -> Option<Vec<Statement>> {
        self.expect(&Token::LBrace);
        let mut stmts = Vec::new();
        while *self.cur() != Token::RBrace && *self.cur() != Token::Eof {
            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            } else {
                self.advance();
            }
        }
        self.expect(&Token::RBrace);
        Some(stmts)
    }

    /// `if condition { ... } else { ... }`
    fn parse_if(&mut self) -> Option<Expression> {
        self.advance(); // skip `if`
        let condition = self.parse_expression(Precedence::Lowest)?;
        let consequence = self.parse_block()?;

        let alternative = if *self.cur() == Token::Else {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        Some(Expression::If {
            condition: Box::new(condition),
            consequence,
            alternative,
        })
    }

    /// `match subject { is Type binding -> stmt ... }`
    fn parse_match(&mut self) -> Option<Expression> {
        self.advance(); // skip `match`
        let subject = self.parse_expression(Precedence::Lowest)?;
        self.expect(&Token::LBrace);

        let mut arms = Vec::new();
        while *self.cur() == Token::Is {
            self.advance(); // skip `is`
            let type_ann = self.parse_single_type()?;
            let binding = self.expect_ident()?;
            self.expect(&Token::Arrow);
            // 分支體：單條語句或區塊
            let body = if *self.cur() == Token::LBrace {
                self.parse_block()?
            } else {
                let stmt = self.parse_statement()?;
                vec![stmt]
            };
            arms.push(MatchArm {
                type_ann,
                binding,
                body,
            });
        }

        self.expect(&Token::RBrace);
        Some(Expression::Match {
            subject: Box::new(subject),
            arms,
        })
    }

    /// `for item in iterable { body }`
    fn parse_for(&mut self) -> Option<Expression> {
        self.advance(); // skip `for`
        let item = self.expect_ident()?;
        self.expect(&Token::In);
        let iterable = self.parse_expression(Precedence::Lowest)?;
        let body = self.parse_block()?;

        Some(Expression::For {
            item,
            iterable: Box::new(iterable),
            body,
        })
    }

    /// `while condition { body }`
    fn parse_while(&mut self) -> Option<Expression> {
        self.advance(); // skip `while`
        let condition = self.parse_expression(Precedence::Lowest)?;
        let body = self.parse_block()?;
        Some(Expression::While {
            condition: Box::new(condition),
            body,
        })
    }

    /// `ray { body }`
    fn parse_ray(&mut self) -> Option<Expression> {
        self.advance(); // skip `ray`
        let body = self.parse_block()?;
        Some(Expression::Ray { body })
    }

    /// `[expr, expr, ...]`
    fn parse_list(&mut self) -> Option<Expression> {
        self.advance(); // skip [
        let items = self.parse_expression_list(Token::RBracket);
        self.expect(&Token::RBracket);
        Some(Expression::ListLiteral(items))
    }

    /// `{key: value, ...}`
    fn parse_map(&mut self) -> Option<Expression> {
        self.advance(); // skip {
        let mut pairs = Vec::new();
        while *self.cur() != Token::RBrace && *self.cur() != Token::Eof {
            let key = self.parse_expression(Precedence::Lowest)?;
            self.expect(&Token::Colon);
            let value = self.parse_expression(Precedence::Lowest)?;
            pairs.push((key, value));
            if *self.cur() == Token::Comma {
                self.advance();
            }
        }
        self.expect(&Token::RBrace);
        Some(Expression::MapLiteral(pairs))
    }

    /// `Shadow(expr)`
    fn parse_shadow_constructor(&mut self) -> Option<Expression> {
        self.advance(); // skip `Shadow`
        self.expect(&Token::LParen);
        let inner = self.parse_expression(Precedence::Lowest)?;
        self.expect(&Token::RParen);
        Some(Expression::ShadowLiteral(Box::new(inner)))
    }

    /// 函數呼叫: `expr(args)`
    fn parse_call(&mut self, function: Expression) -> Option<Expression> {
        self.advance(); // skip (
        let args = self.parse_expression_list(Token::RParen);
        self.expect(&Token::RParen);
        Some(Expression::Call {
            function: Box::new(function),
            args,
        })
    }

    /// 索引: `expr[index]`
    fn parse_index(&mut self, left: Expression) -> Option<Expression> {
        self.advance(); // skip [
        let index = self.parse_expression(Precedence::Lowest)?;
        self.expect(&Token::RBracket);
        Some(Expression::Index {
            left: Box::new(left),
            index: Box::new(index),
        })
    }

    /// Range: `start..end`
    fn parse_range(&mut self, left: Expression) -> Option<Expression> {
        self.advance(); // skip `..`
        let right = self.parse_expression(Precedence::Sum)?;
        Some(Expression::Range {
            start: Box::new(left),
            end: Box::new(right),
        })
    }

    /// 屬性存取: `expr.field`
    fn parse_dot(&mut self, left: Expression) -> Option<Expression> {
        self.advance(); // skip .
        let field = self.expect_ident()?;
        Some(Expression::Dot {
            left: Box::new(left),
            field,
        })
    }

    /// 解析逗號分隔的表達式列表
    fn parse_expression_list(&mut self, end: Token) -> Vec<Expression> {
        let mut list = Vec::new();
        if *self.cur() == end {
            return list;
        }
        if let Some(expr) = self.parse_expression(Precedence::Lowest) {
            list.push(expr);
        }
        while *self.cur() == Token::Comma {
            self.advance();
            if *self.cur() == end {
                break;
            }
            if let Some(expr) = self.parse_expression(Precedence::Lowest) {
                list.push(expr);
            }
        }
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Program {
        let mut parser = Parser::new(input);
        let program = parser.parse();
        if !parser.errors.is_empty() {
            panic!("parser errors: {:?}", parser.errors);
        }
        program
    }

    fn parse_expr(input: &str) -> Expression {
        let program = parse(input);
        match program.statements.into_iter().next().unwrap() {
            Statement::Expression(expr) => expr,
            other => panic!("expected expression statement, got {:?}", other),
        }
    }

    // ── 變數綁定 ──

    #[test]
    fn test_lit_statement() {
        let program = parse("lit x = 42");
        assert_eq!(
            program.statements[0],
            Statement::Lit {
                name: "x".to_string(),
                value: Expression::IntLiteral(42),
            }
        );
    }

    #[test]
    fn test_glow_statement() {
        let program = parse("glow count = 0");
        assert_eq!(
            program.statements[0],
            Statement::Glow {
                name: "count".to_string(),
                value: Expression::IntLiteral(0),
            }
        );
    }

    #[test]
    fn test_assign_statement() {
        let program = parse("count = 10");
        assert_eq!(
            program.statements[0],
            Statement::Assign {
                name: "count".to_string(),
                value: Expression::IntLiteral(10),
            }
        );
    }

    // ── output ──

    #[test]
    fn test_out_statement() {
        let program = parse(r#"out "hello""#);
        assert_eq!(
            program.statements[0],
            Statement::Out(Expression::StringLiteral("hello".to_string()))
        );
    }

    // ── 字面量 ──

    #[test]
    fn test_literals() {
        assert_eq!(parse_expr("42"), Expression::IntLiteral(42));
        assert_eq!(parse_expr("3.14"), Expression::FloatLiteral(3.14));
        assert_eq!(
            parse_expr(r#""hello""#),
            Expression::StringLiteral("hello".to_string())
        );
        assert_eq!(parse_expr("true"), Expression::BoolLiteral(true));
        assert_eq!(parse_expr("false"), Expression::BoolLiteral(false));
    }

    // ── 前綴 ──

    #[test]
    fn test_prefix() {
        let expr = parse_expr("-5");
        assert_eq!(
            expr,
            Expression::Prefix {
                operator: PrefixOp::Negate,
                right: Box::new(Expression::IntLiteral(5)),
            }
        );

        let expr = parse_expr("!true");
        assert_eq!(
            expr,
            Expression::Prefix {
                operator: PrefixOp::Bang,
                right: Box::new(Expression::BoolLiteral(true)),
            }
        );

        let expr = parse_expr("not false");
        assert_eq!(
            expr,
            Expression::Prefix {
                operator: PrefixOp::Not,
                right: Box::new(Expression::BoolLiteral(false)),
            }
        );
    }

    // ── 中綴運算 ──

    #[test]
    fn test_infix_arithmetic() {
        let expr = parse_expr("1 + 2");
        assert_eq!(
            expr,
            Expression::Infix {
                left: Box::new(Expression::IntLiteral(1)),
                operator: InfixOp::Add,
                right: Box::new(Expression::IntLiteral(2)),
            }
        );
    }

    #[test]
    fn test_operator_precedence() {
        // 1 + 2 * 3 → 1 + (2 * 3)
        let expr = parse_expr("1 + 2 * 3");
        assert_eq!(
            expr,
            Expression::Infix {
                left: Box::new(Expression::IntLiteral(1)),
                operator: InfixOp::Add,
                right: Box::new(Expression::Infix {
                    left: Box::new(Expression::IntLiteral(2)),
                    operator: InfixOp::Mul,
                    right: Box::new(Expression::IntLiteral(3)),
                }),
            }
        );
    }

    #[test]
    fn test_grouped_expression() {
        // (1 + 2) * 3
        let expr = parse_expr("(1 + 2) * 3");
        assert_eq!(
            expr,
            Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::IntLiteral(1)),
                    operator: InfixOp::Add,
                    right: Box::new(Expression::IntLiteral(2)),
                }),
                operator: InfixOp::Mul,
                right: Box::new(Expression::IntLiteral(3)),
            }
        );
    }

    #[test]
    fn test_comparison_and_logic() {
        let expr = parse_expr("x > 0 and y < 10");
        match expr {
            Expression::Infix { operator, .. } => assert_eq!(operator, InfixOp::And),
            _ => panic!("expected infix"),
        }
    }

    // ── 函數 ──

    #[test]
    fn test_function() {
        let expr = parse_expr("fn add(a: Int, b: Int) -> Int { out a + b }");
        match expr {
            Expression::Function {
                name,
                params,
                return_type,
                body,
            } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "a");
                assert_eq!(params[0].type_ann, TypeAnnotation::Int);
                assert_eq!(return_type, Some(TypeAnnotation::Int));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_function_union_return() {
        let expr = parse_expr(
            r#"fn dataShow(id: Int) -> String | Shadow { out "ok" }"#,
        );
        match expr {
            Expression::Function { return_type, .. } => {
                assert_eq!(
                    return_type,
                    Some(TypeAnnotation::Union(vec![
                        TypeAnnotation::Str,
                        TypeAnnotation::Shadow,
                    ]))
                );
            }
            _ => panic!("expected function"),
        }
    }

    // ── 函數呼叫 ──

    #[test]
    fn test_call() {
        let expr = parse_expr("add(1, 2)");
        match expr {
            Expression::Call { function, args } => {
                assert_eq!(*function, Expression::Ident("add".to_string()));
                assert_eq!(args.len(), 2);
            }
            _ => panic!("expected call"),
        }
    }

    // ── if / else ──

    #[test]
    fn test_if_else() {
        let expr = parse_expr(r#"if x > 0 { out "yes" } else { out "no" }"#);
        match expr {
            Expression::If {
                consequence,
                alternative,
                ..
            } => {
                assert_eq!(consequence.len(), 1);
                assert!(alternative.is_some());
                assert_eq!(alternative.unwrap().len(), 1);
            }
            _ => panic!("expected if"),
        }
    }

    #[test]
    fn test_if_no_else() {
        let expr = parse_expr(r#"if active { out "on" }"#);
        match expr {
            Expression::If { alternative, .. } => {
                assert!(alternative.is_none());
            }
            _ => panic!("expected if"),
        }
    }

    // ── match ──

    #[test]
    fn test_match() {
        let input = r#"match result {
            is String val -> print(val)
            is Shadow s -> print(s.message)
        }"#;
        let expr = parse_expr(input);
        match expr {
            Expression::Match { arms, .. } => {
                assert_eq!(arms.len(), 2);
                assert_eq!(arms[0].type_ann, TypeAnnotation::Str);
                assert_eq!(arms[0].binding, "val");
                assert_eq!(arms[1].type_ann, TypeAnnotation::Shadow);
                assert_eq!(arms[1].binding, "s");
            }
            _ => panic!("expected match"),
        }
    }

    // ── for ──

    #[test]
    fn test_for() {
        let expr = parse_expr("for item in items { print(item) }");
        match expr {
            Expression::For { item, .. } => {
                assert_eq!(item, "item");
            }
            _ => panic!("expected for"),
        }
    }

    // ── List / Map ──

    #[test]
    fn test_list() {
        let expr = parse_expr("[1, 2, 3]");
        match expr {
            Expression::ListLiteral(items) => assert_eq!(items.len(), 3),
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn test_map() {
        let expr = parse_expr(r#"{"name": "Sunny", "version": 1}"#);
        match expr {
            Expression::MapLiteral(pairs) => assert_eq!(pairs.len(), 2),
            _ => panic!("expected map"),
        }
    }

    // ── 索引與屬性存取 ──

    #[test]
    fn test_index() {
        let expr = parse_expr("list[0]");
        match expr {
            Expression::Index { left, index } => {
                assert_eq!(*left, Expression::Ident("list".to_string()));
                assert_eq!(*index, Expression::IntLiteral(0));
            }
            _ => panic!("expected index"),
        }
    }

    #[test]
    fn test_dot() {
        let expr = parse_expr("user.name");
        match expr {
            Expression::Dot { left, field } => {
                assert_eq!(*left, Expression::Ident("user".to_string()));
                assert_eq!(field, "name");
            }
            _ => panic!("expected dot"),
        }
    }

    // ── Shadow ──

    #[test]
    fn test_shadow_constructor() {
        let expr = parse_expr(r#"Shadow("not found")"#);
        match expr {
            Expression::ShadowLiteral(inner) => {
                assert_eq!(
                    *inner,
                    Expression::StringLiteral("not found".to_string())
                );
            }
            _ => panic!("expected shadow"),
        }
    }

    // ── ray ──

    #[test]
    fn test_ray() {
        let expr = parse_expr("ray { print(1) }");
        match expr {
            Expression::Ray { body } => {
                assert_eq!(body.len(), 1);
            }
            _ => panic!("expected ray"),
        }
    }

    // ── 整合測試：來自 SPEC.md 的真實範例 ──

    #[test]
    fn test_full_function_from_spec() {
        let input = r#"fn dataShow(id: Int) -> String | Shadow {
    if id < 0 {
        out Shadow("ID must be positive")
    }
    out "Valid Data"
}"#;
        let expr = parse_expr(input);
        match expr {
            Expression::Function {
                name,
                params,
                return_type,
                body,
            } => {
                assert_eq!(name, "dataShow");
                assert_eq!(params.len(), 1);
                assert_eq!(
                    return_type,
                    Some(TypeAnnotation::Union(vec![
                        TypeAnnotation::Str,
                        TypeAnnotation::Shadow,
                    ]))
                );
                assert_eq!(body.len(), 2); // if + output
            }
            _ => panic!("expected function"),
        }
    }

    // ── Range ──

    #[test]
    fn test_range() {
        let expr = parse_expr("0..10");
        match expr {
            Expression::Range { start, end } => {
                assert_eq!(*start, Expression::IntLiteral(0));
                assert_eq!(*end, Expression::IntLiteral(10));
            }
            _ => panic!("expected range"),
        }
    }

    #[test]
    fn test_for_range() {
        let expr = parse_expr("for i in 0..5 { print(i) }");
        match expr {
            Expression::For { item, iterable, .. } => {
                assert_eq!(item, "i");
                match *iterable {
                    Expression::Range { .. } => {}
                    _ => panic!("expected range iterable"),
                }
            }
            _ => panic!("expected for"),
        }
    }

    // ── 匿名函數 ──

    #[test]
    fn test_anonymous_function() {
        let program = parse("lit add = fn(x, y) { out x + y }");
        match &program.statements[0] {
            Statement::Lit { name, value } => {
                assert_eq!(name, "add");
                match value {
                    Expression::Function { name, params, .. } => {
                        assert_eq!(name, "");
                        assert_eq!(params.len(), 2);
                        assert_eq!(params[0].name, "x");
                        assert_eq!(params[1].name, "y");
                    }
                    _ => panic!("expected function"),
                }
            }
            _ => panic!("expected lit"),
        }
    }

    // ── Import ──

    #[test]
    fn test_import() {
        let program = parse(r#"import "utils.sunny""#);
        match &program.statements[0] {
            Statement::Import { path } => {
                assert_eq!(path, "utils.sunny");
            }
            _ => panic!("expected import"),
        }
    }
}

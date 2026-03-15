use std::fmt;

/// Sunny 語言的所有 Token 類型定義
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ── 字面量 (Literals) ──
    Int(i64),
    Float(f64),
    StringLiteral(String),
    Bool(bool),
    Ident(String),

    // ── 關鍵字 (Keywords) ──
    Lit,       // lit  — 不可變綁定
    Glow,      // glow — 可變綁定
    Fn,        // fn   — 函數定義
    Out,       // out — 回傳值
    Match,     // match  — 模式匹配
    Is,        // is     — 匹配分支
    If,        // if     — 條件判斷
    Else,      // else — else 分支
    For,       // for  — 迴圈
    In,        // in     — 迴圈迭代
    Ray,       // ray    — 併發任務
    Import,    // import — 模組匯入
    While,     // while  — 條件迴圈

    // ── 邏輯關鍵字 (Logical Keywords) ──
    And, // and
    Or,  // or
    Not, // not

    // ── 型別關鍵字 (Type Keywords) ──
    TypeInt,    // Int
    TypeFloat,  // Float
    TypeString, // String
    TypeBool,   // Bool
    TypeList,   // List
    TypeMap,    // Map
    TypeShadow, // Shadow

    // ── 算術運算子 (Arithmetic Operators) ──
    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /
    Percent, // %

    // ── 比較運算子 (Comparison Operators) ──
    Equal,        // ==
    NotEqual,     // !=
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=

    // ── 邏輯符號運算子 (Logical Symbol Operators) ──
    AmpAmp, // &&
    PipePipe, // ||
    Bang,   // !

    // ── 賦值 (Assignment) ──
    Assign, // =

    // ── 分隔符號 (Delimiters) ──
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    Comma,    // ,
    Colon,    // :
    Dot,      // .
    DotDot,   // ..
    Arrow,    // ->
    Pipe,     // |

    // ── 特殊 (Special) ──
    Eof,
    Illegal(char),
}

impl Token {
    /// 從識別符字串查找是否為關鍵字，若不是則回傳 Ident
    pub fn lookup_ident(ident: &str) -> Token {
        match ident {
            // 關鍵字
            "lit" => Token::Lit,
            "glow" => Token::Glow,
            "fn" => Token::Fn,
            "out" => Token::Out,
            "match" => Token::Match,
            "is" => Token::Is,
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "in" => Token::In,
            "ray" => Token::Ray,
            "import" => Token::Import,
            "while" => Token::While,

            // 邏輯
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,

            // 布林字面量
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),

            // 型別
            "Int" => Token::TypeInt,
            "Float" => Token::TypeFloat,
            "String" => Token::TypeString,
            "Bool" => Token::TypeBool,
            "List" => Token::TypeList,
            "Map" => Token::TypeMap,
            "Shadow" => Token::TypeShadow,

            // 非關鍵字 → 識別符
            _ => Token::Ident(ident.to_string()),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // 字面量
            Token::Int(v) => write!(f, "{}", v),
            Token::Float(v) => write!(f, "{}", v),
            Token::StringLiteral(v) => write!(f, "\"{}\"", v),
            Token::Bool(v) => write!(f, "{}", v),
            Token::Ident(v) => write!(f, "{}", v),

            // 關鍵字
            Token::Lit => write!(f, "lit"),
            Token::Glow => write!(f, "glow"),
            Token::Fn => write!(f, "fn"),
            Token::Out => write!(f, "out"),
            Token::Match => write!(f, "match"),
            Token::Is => write!(f, "is"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::For => write!(f, "for"),
            Token::In => write!(f, "in"),
            Token::Ray => write!(f, "ray"),
            Token::Import => write!(f, "import"),
            Token::While => write!(f, "while"),

            // 邏輯關鍵字
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),

            // 型別
            Token::TypeInt => write!(f, "Int"),
            Token::TypeFloat => write!(f, "Float"),
            Token::TypeString => write!(f, "String"),
            Token::TypeBool => write!(f, "Bool"),
            Token::TypeList => write!(f, "List"),
            Token::TypeMap => write!(f, "Map"),
            Token::TypeShadow => write!(f, "Shadow"),

            // 運算子
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::LessThan => write!(f, "<"),
            Token::GreaterThan => write!(f, ">"),
            Token::LessEqual => write!(f, "<="),
            Token::GreaterEqual => write!(f, ">="),
            Token::AmpAmp => write!(f, "&&"),
            Token::PipePipe => write!(f, "||"),
            Token::Bang => write!(f, "!"),
            Token::Assign => write!(f, "="),

            // 分隔符號
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Dot => write!(f, "."),
            Token::DotDot => write!(f, ".."),
            Token::Arrow => write!(f, "->"),
            Token::Pipe => write!(f, "|"),

            // 特殊
            Token::Eof => write!(f, "EOF"),
            Token::Illegal(c) => write!(f, "ILLEGAL({})", c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_keywords() {
        assert_eq!(Token::lookup_ident("lit"), Token::Lit);
        assert_eq!(Token::lookup_ident("glow"), Token::Glow);
        assert_eq!(Token::lookup_ident("fn"), Token::Fn);
        assert_eq!(Token::lookup_ident("out"), Token::Out);
        assert_eq!(Token::lookup_ident("match"), Token::Match);
        assert_eq!(Token::lookup_ident("else"), Token::Else);
        assert_eq!(Token::lookup_ident("for"), Token::For);
        assert_eq!(Token::lookup_ident("in"), Token::In);
        assert_eq!(Token::lookup_ident("ray"), Token::Ray);
        assert_eq!(Token::lookup_ident("true"), Token::Bool(true));
        assert_eq!(Token::lookup_ident("false"), Token::Bool(false));
        assert_eq!(Token::lookup_ident("and"), Token::And);
        assert_eq!(Token::lookup_ident("or"), Token::Or);
        assert_eq!(Token::lookup_ident("not"), Token::Not);
    }

    #[test]
    fn test_lookup_types() {
        assert_eq!(Token::lookup_ident("Int"), Token::TypeInt);
        assert_eq!(Token::lookup_ident("Float"), Token::TypeFloat);
        assert_eq!(Token::lookup_ident("String"), Token::TypeString);
        assert_eq!(Token::lookup_ident("Bool"), Token::TypeBool);
        assert_eq!(Token::lookup_ident("List"), Token::TypeList);
        assert_eq!(Token::lookup_ident("Map"), Token::TypeMap);
        assert_eq!(Token::lookup_ident("Shadow"), Token::TypeShadow);
    }

    #[test]
    fn test_lookup_ident_returns_ident_for_unknown() {
        assert_eq!(
            Token::lookup_ident("userName"),
            Token::Ident("userName".to_string())
        );
        assert_eq!(
            Token::lookup_ident("orderStore"),
            Token::Ident("orderStore".to_string())
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Token::Lit), "lit");
        assert_eq!(format!("{}", Token::Arrow), "->");
        assert_eq!(format!("{}", Token::Int(42)), "42");
        assert_eq!(format!("{}", Token::StringLiteral("hello".into())), "\"hello\"");
        assert_eq!(format!("{}", Token::Illegal('$')), "ILLEGAL($)");
    }
}

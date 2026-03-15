use crate::token::Token;

/// Sunny 語言詞法分析器
/// 將原始碼字串逐字元掃描，產出 Token 流
pub struct Lexer {
    input: Vec<char>,
    pos: usize,      // 當前字元位置
    next_pos: usize,  // 下一個字元位置
    ch: char,         // 當前字元
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer {
            input: input.chars().collect(),
            pos: 0,
            next_pos: 0,
            ch: '\0',
        };
        lexer.read_char();
        lexer
    }

    /// 讀取下一個字元，推進游標
    fn read_char(&mut self) {
        if self.next_pos >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.next_pos];
        }
        self.pos = self.next_pos;
        self.next_pos += 1;
    }

    /// 偷看下一個字元但不推進游標
    fn peek_char(&self) -> char {
        if self.next_pos >= self.input.len() {
            '\0'
        } else {
            self.input[self.next_pos]
        }
    }

    /// 掃描並回傳下一個 Token
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        self.skip_comments();
        // 註解跳過後可能又遇到空白，再跳一次
        self.skip_whitespace();

        let token = match self.ch {
            // ── 運算子與符號 ──
            '+' => Token::Plus,
            '*' => Token::Star,
            '%' => Token::Percent,
            ',' => Token::Comma,
            ':' => Token::Colon,
            '.' => {
                if self.peek_char() == '.' {
                    self.read_char();
                    Token::DotDot
                } else {
                    Token::Dot
                }
            }
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,

            // 可能是 -> 或單獨的 -
            '-' => {
                if self.peek_char() == '>' {
                    self.read_char();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }

            // 可能是 / 或註解（已被 skip_comments 處理，這裡只剩真正的除法）
            '/' => Token::Slash,

            // = 或 ==
            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::Equal
                } else {
                    Token::Assign
                }
            }

            // ! 或 !=
            '!' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::NotEqual
                } else {
                    Token::Bang
                }
            }

            // < 或 <=
            '<' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::LessEqual
                } else {
                    Token::LessThan
                }
            }

            // > 或 >=
            '>' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::GreaterEqual
                } else {
                    Token::GreaterThan
                }
            }

            // &&
            '&' => {
                if self.peek_char() == '&' {
                    self.read_char();
                    Token::AmpAmp
                } else {
                    Token::Illegal('&')
                }
            }

            // || 或 |
            '|' => {
                if self.peek_char() == '|' {
                    self.read_char();
                    Token::PipePipe
                } else {
                    Token::Pipe
                }
            }

            // ── 字串字面量（雙引號或單引號）──
            '"' | '\'' => {
                return self.read_string();
            }

            // ── EOF ──
            '\0' => Token::Eof,

            // ── 識別符、數字、或非法字元 ──
            _ => {
                if is_letter(self.ch) {
                    return self.read_identifier();
                } else if self.ch.is_ascii_digit() {
                    return self.read_number();
                } else {
                    Token::Illegal(self.ch)
                }
            }
        };

        self.read_char();
        token
    }

    /// 跳過空白字元（空格、Tab、換行、回車）
    fn skip_whitespace(&mut self) {
        while self.ch == ' ' || self.ch == '\t' || self.ch == '\n' || self.ch == '\r' {
            self.read_char();
        }
    }

    /// 跳過註解：// 單行 和 /* */ 多行（迭代式，避免 stack overflow）
    fn skip_comments(&mut self) {
        loop {
            if self.ch == '/' && self.peek_char() == '/' {
                // 單行註解：跳到行尾
                while self.ch != '\n' && self.ch != '\0' {
                    self.read_char();
                }
                self.skip_whitespace();
                // 繼續迴圈檢查是否還有註解
            } else if self.ch == '/' && self.peek_char() == '*' {
                // 多行註解：找到 */
                self.read_char(); // 跳過 /
                self.read_char(); // 跳過 *
                loop {
                    if self.ch == '\0' {
                        break;
                    }
                    if self.ch == '*' && self.peek_char() == '/' {
                        self.read_char(); // 跳過 *
                        self.read_char(); // 跳過 /
                        break;
                    }
                    self.read_char();
                }
                self.skip_whitespace();
                // 繼續迴圈檢查是否還有註解
            } else {
                break;
            }
        }
    }

    /// 讀取識別符或關鍵字
    fn read_identifier(&mut self) -> Token {
        let start = self.pos;
        while is_letter(self.ch) || self.ch.is_ascii_digit() {
            self.read_char();
        }
        let literal: String = self.input[start..self.pos].iter().collect();
        Token::lookup_ident(&literal)
    }

    /// 讀取數字（整數或浮點數）
    fn read_number(&mut self) -> Token {
        let start = self.pos;
        while self.ch.is_ascii_digit() {
            self.read_char();
        }

        // 檢查是否為浮點數
        if self.ch == '.' && self.peek_char().is_ascii_digit() {
            self.read_char(); // 跳過小數點
            while self.ch.is_ascii_digit() {
                self.read_char();
            }
            let literal: String = self.input[start..self.pos].iter().collect();
            match literal.parse() {
                Ok(v) => Token::Float(v),
                Err(_) => Token::Illegal('0'), // 數字溢位
            }
        } else {
            let literal: String = self.input[start..self.pos].iter().collect();
            match literal.parse() {
                Ok(v) => Token::Int(v),
                Err(_) => Token::Illegal('0'), // 數字溢位
            }
        }
    }

    /// 讀取字串字面量（支援雙引號與單引號）
    fn read_string(&mut self) -> Token {
        let quote = self.ch;
        self.read_char(); // 跳過開頭引號
        let mut literal = String::new();

        loop {
            match self.ch {
                '\0' | '\n' => break, // 未關閉的字串
                c if c == quote => break,
                '\\' => {
                    self.read_char(); // 跳過反斜線
                    match self.ch {
                        'n' => literal.push('\n'),
                        't' => literal.push('\t'),
                        'r' => literal.push('\r'),
                        '\\' => literal.push('\\'),
                        '\'' => literal.push('\''),
                        '"' => literal.push('"'),
                        '0' => literal.push('\0'),
                        other => {
                            literal.push('\\');
                            literal.push(other);
                        }
                    }
                    self.read_char();
                    continue;
                }
                c => literal.push(c),
            }
            self.read_char();
        }

        self.read_char(); // 跳過結尾引號
        Token::StringLiteral(literal)
    }

    /// 便利方法：將整個輸入轉為 Token 列表
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            if tok == Token::Eof {
                tokens.push(tok);
                break;
            }
            tokens.push(tok);
        }
        tokens
    }
}

/// 判斷字元是否為合法識別符開頭/組成（字母、底線）
fn is_letter(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_symbols() {
        let input = "+-*/%";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Minus);
        assert_eq!(lexer.next_token(), Token::Star);
        assert_eq!(lexer.next_token(), Token::Slash);
        assert_eq!(lexer.next_token(), Token::Percent);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_two_char_operators() {
        let input = "== != <= >= -> && ||";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::NotEqual);
        assert_eq!(lexer.next_token(), Token::LessEqual);
        assert_eq!(lexer.next_token(), Token::GreaterEqual);
        assert_eq!(lexer.next_token(), Token::Arrow);
        assert_eq!(lexer.next_token(), Token::AmpAmp);
        assert_eq!(lexer.next_token(), Token::PipePipe);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_delimiters() {
        let input = "(){}[],:.|";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::LBrace);
        assert_eq!(lexer.next_token(), Token::RBrace);
        assert_eq!(lexer.next_token(), Token::LBracket);
        assert_eq!(lexer.next_token(), Token::RBracket);
        assert_eq!(lexer.next_token(), Token::Comma);
        assert_eq!(lexer.next_token(), Token::Colon);
        assert_eq!(lexer.next_token(), Token::Dot);
        assert_eq!(lexer.next_token(), Token::Pipe);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_keywords_and_idents() {
        let input = "lit glow fn out match is if else for in ray";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Lit);
        assert_eq!(lexer.next_token(), Token::Glow);
        assert_eq!(lexer.next_token(), Token::Fn);
        assert_eq!(lexer.next_token(), Token::Out);
        assert_eq!(lexer.next_token(), Token::Match);
        assert_eq!(lexer.next_token(), Token::Is);
        assert_eq!(lexer.next_token(), Token::If);
        assert_eq!(lexer.next_token(), Token::Else);
        assert_eq!(lexer.next_token(), Token::For);
        assert_eq!(lexer.next_token(), Token::In);
        assert_eq!(lexer.next_token(), Token::Ray);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_logical_keywords() {
        let input = "true and false or not";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Bool(true));
        assert_eq!(lexer.next_token(), Token::And);
        assert_eq!(lexer.next_token(), Token::Bool(false));
        assert_eq!(lexer.next_token(), Token::Or);
        assert_eq!(lexer.next_token(), Token::Not);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_type_keywords() {
        let input = "Int Float String Bool List Map Shadow";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::TypeInt);
        assert_eq!(lexer.next_token(), Token::TypeFloat);
        assert_eq!(lexer.next_token(), Token::TypeString);
        assert_eq!(lexer.next_token(), Token::TypeBool);
        assert_eq!(lexer.next_token(), Token::TypeList);
        assert_eq!(lexer.next_token(), Token::TypeMap);
        assert_eq!(lexer.next_token(), Token::TypeShadow);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_numbers() {
        let input = "42 3.14 100";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Int(42));
        assert_eq!(lexer.next_token(), Token::Float(3.14));
        assert_eq!(lexer.next_token(), Token::Int(100));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_strings_double_quote() {
        let input = r#""hello world""#;
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token(),
            Token::StringLiteral("hello world".to_string())
        );
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_strings_single_quote() {
        let input = "'sunny lang'";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token(),
            Token::StringLiteral("sunny lang".to_string())
        );
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_single_line_comment() {
        let input = "lit x = 10 // this is a comment\nlit y = 20";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Lit);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Int(10));
        assert_eq!(lexer.next_token(), Token::Lit);
        assert_eq!(lexer.next_token(), Token::Ident("y".to_string()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Int(20));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_multi_line_comment() {
        let input = "lit x = /* skip this */ 10";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Lit);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Int(10));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_sunny_function() {
        // 來自 SPEC.md 的真實範例
        let input = r#"fn bookShow(id: Int) -> String | Shadow {
    lit bookName = "Sunny Guide"
    out bookName
}"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0], Token::Fn);
        assert_eq!(tokens[1], Token::Ident("bookShow".to_string()));
        assert_eq!(tokens[2], Token::LParen);
        assert_eq!(tokens[3], Token::Ident("id".to_string()));
        assert_eq!(tokens[4], Token::Colon);
        assert_eq!(tokens[5], Token::TypeInt);
        assert_eq!(tokens[6], Token::RParen);
        assert_eq!(tokens[7], Token::Arrow);
        assert_eq!(tokens[8], Token::TypeString);
        assert_eq!(tokens[9], Token::Pipe);
        assert_eq!(tokens[10], Token::TypeShadow);
        assert_eq!(tokens[11], Token::LBrace);
        assert_eq!(tokens[12], Token::Lit);
        assert_eq!(tokens[13], Token::Ident("bookName".to_string()));
        assert_eq!(tokens[14], Token::Assign);
        assert_eq!(tokens[15], Token::StringLiteral("Sunny Guide".to_string()));
        assert_eq!(tokens[16], Token::Out);
        assert_eq!(tokens[17], Token::Ident("bookName".to_string()));
        assert_eq!(tokens[18], Token::RBrace);
        assert_eq!(tokens[19], Token::Eof);
    }

    #[test]
    fn test_camelcase_ident() {
        let input = "userStatus orderStore productIndex";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("userStatus".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("orderStore".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("productIndex".to_string()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_dot_dot_range() {
        let input = "0..10";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Int(0));
        assert_eq!(lexer.next_token(), Token::DotDot);
        assert_eq!(lexer.next_token(), Token::Int(10));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_dot_vs_dotdot() {
        let input = "a.b 0..5";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("a".to_string()));
        assert_eq!(lexer.next_token(), Token::Dot);
        assert_eq!(lexer.next_token(), Token::Ident("b".to_string()));
        assert_eq!(lexer.next_token(), Token::Int(0));
        assert_eq!(lexer.next_token(), Token::DotDot);
        assert_eq!(lexer.next_token(), Token::Int(5));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_import_keyword() {
        let input = r#"import "utils.sunny""#;
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Import);
        assert_eq!(
            lexer.next_token(),
            Token::StringLiteral("utils.sunny".to_string())
        );
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}

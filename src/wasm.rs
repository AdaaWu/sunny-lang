use wasm_bindgen::prelude::*;

use crate::environment::Environment;
use crate::evaluator::{EvalResult, Evaluator};
use crate::parser::Parser;

/// 執行 Sunny 程式碼，回傳 JSON 結果
/// { "output": [...], "result": "...", "error": null }
#[wasm_bindgen]
pub fn eval_sunny(source: &str) -> String {
    let mut parser = Parser::new(source);
    let program = parser.parse();

    if !parser.errors.is_empty() {
        let errors: Vec<String> = parser.errors.iter().map(|e| format!("\"{}\"", e)).collect();
        return format!(
            "{{\"output\": [], \"result\": null, \"error\": [{}]}}",
            errors.join(", ")
        );
    }

    let mut evaluator = Evaluator::new();
    let mut env = Environment::new();
    let result = evaluator.eval_program(&program, &mut env);

    let output: Vec<String> = evaluator
        .output_buffer
        .iter()
        .map(|s| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect();

    let (result_str, error_str) = match result {
        EvalResult::Val(val) => (format!("\"{}\"", val), "null".to_string()),
        EvalResult::Output(val) => (format!("\"{}\"", val), "null".to_string()),
        EvalResult::Err(msg) => ("null".to_string(), format!("\"{}\"", msg)),
    };

    format!(
        "{{\"output\": [{}], \"result\": {}, \"error\": {}}}",
        output.join(", "),
        result_str,
        error_str
    )
}

/// Tokenize Sunny 程式碼，回傳 JSON token 列表（供語法高亮用）
#[wasm_bindgen]
pub fn tokenize_sunny(source: &str) -> String {
    let mut lexer = crate::lexer::Lexer::new(source);
    let tokens = lexer.tokenize();

    let parts: Vec<String> = tokens
        .iter()
        .map(|t| {
            let kind = match t {
                crate::token::Token::Int(_) | crate::token::Token::Float(_) => "number",
                crate::token::Token::StringLiteral(_) => "string",
                crate::token::Token::Bool(_) => "keyword",
                crate::token::Token::Ident(name) => {
                    match name.as_str() {
                        "print" | "await" | "read_file" | "render_md" | "render_template"
                        | "write_file" | "len" | "type_of" | "to_int" | "to_float"
                        | "to_string" | "json_encode" | "time_now" => "builtin",
                        _ => "ident",
                    }
                }
                crate::token::Token::Lit
                | crate::token::Token::Glow
                | crate::token::Token::Fn
                | crate::token::Token::Out
                | crate::token::Token::If
                | crate::token::Token::Else
                | crate::token::Token::For
                | crate::token::Token::In
                | crate::token::Token::While
                | crate::token::Token::Match
                | crate::token::Token::Is
                | crate::token::Token::Ray
                | crate::token::Token::Import
                | crate::token::Token::And
                | crate::token::Token::Or
                | crate::token::Token::Not => "keyword",
                crate::token::Token::TypeInt
                | crate::token::Token::TypeFloat
                | crate::token::Token::TypeString
                | crate::token::Token::TypeBool
                | crate::token::Token::TypeList
                | crate::token::Token::TypeMap
                | crate::token::Token::TypeShadow => "type",
                crate::token::Token::Eof => "eof",
                crate::token::Token::Illegal(_) => "error",
                _ => "operator",
            };
            format!("[\"{}\", \"{}\"]", kind, format!("{}", t).replace('"', "\\\""))
        })
        .collect();

    format!("[{}]", parts.join(", "))
}

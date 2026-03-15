use std::env;
use std::fs;
use std::io::{self, Write};

use sunny_lang::environment::{Environment, Value};
use sunny_lang::evaluator::{EvalResult, Evaluator};
use sunny_lang::linter;
use sunny_lang::parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("serve") => {
            let path = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
                eprintln!("Usage: sun serve <file.sunny> [port]");
                std::process::exit(1);
            });
            let port: u16 = args
                .get(3)
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000);
            let source = read_file(path);
            sunny_lang::server::start(&source, port);
        }
        Some("restart") => {
            let path = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
                eprintln!("Usage: sun restart <file.sunny> [port]");
                std::process::exit(1);
            });
            let port: u16 = args
                .get(3)
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000);
            kill_port(port);
            let source = read_file(path);
            sunny_lang::server::start(&source, port);
        }
        Some("stop") => {
            let port: u16 = args
                .get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000);
            kill_port(port);
            println!("Stopped server on port {}", port);
        }
        Some(path) => {
            run_file(path);
        }
        None => {
            repl();
        }
    }
}

/// 殺掉佔用指定 port 的程序（macOS / Linux）
fn kill_port(port: u16) {
    use std::process::Command;

    let output = Command::new("lsof")
        .args(["-ti", &format!(":{}", port)])
        .output();

    if let Ok(out) = output {
        let pids = String::from_utf8_lossy(&out.stdout);
        for pid in pids.split_whitespace() {
            if let Ok(p) = pid.parse::<u32>() {
                if p as u32 != std::process::id() {
                    let _ = Command::new("kill").arg(pid).output();
                    println!("Killed process {} on port {}", pid, port);
                }
            }
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(500));
}

fn read_file(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            std::process::exit(1);
        }
    }
}

/// 執行 .sunny 原始碼檔案
fn run_file(path: &str) {
    let source = read_file(path);

    let mut parser = Parser::new(&source);
    let program = parser.parse();

    if !parser.errors.is_empty() {
        for err in &parser.errors {
            eprintln!("Parse error: {}", err);
        }
        std::process::exit(1);
    }

    let warnings = linter::lint(&program);
    if !warnings.is_empty() {
        for w in &warnings {
            eprintln!("Lint error: {}", w.message);
        }
        std::process::exit(1);
    }

    let mut eval = Evaluator::new();
    let mut env = Environment::new();
    let result = eval.eval_program(&program, &mut env);

    for line in &eval.output_buffer {
        println!("{}", line);
    }

    if let EvalResult::Err(msg) = result {
        eprintln!("Runtime error: {}", msg);
        std::process::exit(1);
    }
}

/// REPL 互動模式
fn repl() {
    println!("Welcome to Sunny Lang! v0.1.0");
    println!("Type your code below. Use Ctrl+D to exit.\n");

    let mut env = Environment::new();

    loop {
        print!("sunny> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => {
                println!("\nGoodbye!");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut parser = Parser::new(trimmed);
        let program = parser.parse();

        if !parser.errors.is_empty() {
            for err in &parser.errors {
                eprintln!("  Parse error: {}", err);
            }
            continue;
        }

        let warnings = linter::lint(&program);
        if !warnings.is_empty() {
            for w in &warnings {
                eprintln!("  Lint: {}", w.message);
            }
            continue;
        }

        let mut eval = Evaluator::new();
        let result = eval.eval_program(&program, &mut env);

        for output in &eval.output_buffer {
            println!("{}", output);
        }

        match &result {
            EvalResult::Val(val) => {
                if *val != Value::Void {
                    println!("{}", val);
                }
            }
            EvalResult::Output(val) => println!("{}", val),
            EvalResult::Err(msg) => eprintln!("  Error: {}", msg),
        }
    }
}

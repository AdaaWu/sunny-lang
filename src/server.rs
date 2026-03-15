use crate::environment::{Environment, Value};
use crate::evaluator::{EvalResult, Evaluator};
use crate::router::{build_routes, match_route, Route};
use tiny_http::{Header, Method, Response, Server};

use std::io::Cursor;

/// 啟動 Sunny HTTP Server
pub fn start(source: &str, port: u16) {
    let mut parser = crate::parser::Parser::new(source);
    let program = parser.parse();
    if !parser.errors.is_empty() {
        eprintln!("Parse errors:");
        for e in &parser.errors {
            eprintln!("  {}", e);
        }
        return;
    }

    // 先執行一次程式，讓全域函數定義進入環境
    let mut evaluator = Evaluator::new();
    let mut env = Environment::new();
    evaluator.eval_program(&program, &mut env);

    // 掃描 AST，自動產生路由表
    let routes = build_routes(&program);

    if routes.is_empty() {
        eprintln!("No routes found. Define functions with Resource Action suffixes:");
        eprintln!("  fn productIndex() -> ...  =>  GET /product");
        eprintln!("  fn productShow(id: Int)   =>  GET /product/:id");
        return;
    }

    println!("Sunny HTTP Server starting on port {}", port);
    println!("Routes:");
    for route in &routes {
        println!("  {} {} -> {}()", route.method, route.path, route.fn_name);
    }
    println!();

    let addr = format!("0.0.0.0:{}", port);
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to start server: {}", e);
            return;
        }
    };

    println!("Listening on http://localhost:{}", port);

    for request in server.incoming_requests() {
        let method_str = match request.method() {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            _ => {
                let _ = request.respond(
                    Response::from_string("Method Not Allowed").with_status_code(405),
                );
                continue;
            }
        };

        let path = request.url().split('?').next().unwrap_or("/").to_string();
        println!("  {} {}", method_str, path);

        let (status, body) = match match_route(&routes, method_str, &path) {
            Some((route, path_params)) => {
                match execute_route(route, &path_params, &env) {
                    RouteResult::Ok(body) => (200, body),
                    RouteResult::Shadow(msg) => {
                        (500, format!("{{\"error\": \"{}\"}}", escape_json(&msg)))
                    }
                    RouteResult::Err(msg) => {
                        (500, format!("{{\"error\": \"{}\"}}", escape_json(&msg)))
                    }
                }
            }
            None if path == "/" => (200, generate_welcome_page(&routes, port)),
            None => (
                404,
                format!("{{\"error\": \"Not Found: {} {}\"}}", method_str, path),
            ),
        };

        // 自動偵測 Content-Type：以 < 開頭視為 HTML，否則 JSON
        let content_type = if body.starts_with('<') {
            "text/html; charset=utf-8"
        } else {
            "application/json; charset=utf-8"
        };
        let header = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap();
        let resp = Response::new(
            tiny_http::StatusCode(status),
            vec![header],
            Cursor::new(body.into_bytes()),
            None,
            None,
        );
        let _ = request.respond(resp);
    }
}

enum RouteResult {
    Ok(String),
    Shadow(String),
    Err(String),
}

fn execute_route(
    route: &Route,
    path_params: &[(String, String)],
    base_env: &Environment,
) -> RouteResult {
    let mut evaluator = Evaluator::new();
    let mut env = Environment::enclosed(base_env.clone());

    // 將路徑參數注入環境（如 id = "42"，嘗試轉 Int）
    for (name, value) in path_params {
        let val = if let Ok(n) = value.parse::<i64>() {
            Value::Int(n)
        } else {
            Value::Str(value.clone())
        };
        let _ = env.define_lit(name, val);
    }

    // 建立引數列表
    let mut args = Vec::new();
    for param in &route.params {
        if let Some(val) = env.get(&param.name) {
            args.push(val);
        } else {
            args.push(Value::Void);
        }
    }

    // 建立函數呼叫環境
    let mut call_env = Environment::enclosed(base_env.clone());
    for (param, arg) in route.params.iter().zip(args.iter()) {
        let _ = call_env.define_lit(&param.name, arg.clone());
    }

    let result = evaluator.eval_block(&route.body, &mut call_env);

    match result {
        EvalResult::Val(val) | EvalResult::Output(val) => match val {
            Value::Shadow(msg) => RouteResult::Shadow(msg),
            // String 直接回傳（支援 HTML），其他型別 JSON 編碼
            Value::Str(s) => RouteResult::Ok(s),
            other => RouteResult::Ok(value_to_json(&other)),
        },
        EvalResult::Err(msg) => RouteResult::Err(msg),
    }
}

fn value_to_json(val: &Value) -> String {
    match val {
        Value::Int(n) => format!("{}", n),
        Value::Float(f) => format!("{}", f),
        Value::Str(s) => format!("\"{}\"", escape_json(s)),
        Value::Bool(b) => format!("{}", b),
        Value::Void => "null".to_string(),
        Value::List(items) => {
            let parts: Vec<String> = items.iter().map(|v| value_to_json(v)).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Map(pairs) => {
            let parts: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", escape_json(k), value_to_json(v)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        Value::Shadow(msg) => format!("{{\"shadow\": \"{}\"}}", escape_json(msg)),
        Value::Function { name, .. } => format!("\"<fn {}>\"", name),
        Value::RayHandle(_) => "\"<ray>\"".to_string(),
    }
}

fn generate_welcome_page(routes: &[Route], port: u16) -> String {
    let mut rows = String::new();
    for route in routes {
        let method_color = match route.method {
            crate::router::HttpMethod::Get => "#61affe",
            crate::router::HttpMethod::Post => "#49cc90",
            crate::router::HttpMethod::Put => "#fca130",
            crate::router::HttpMethod::Delete => "#f93e3e",
        };
        // 產生範例路徑（把 :id 替換成 1）
        let example_path = route.path.replace(":id", "1");
        let curl_cmd = format!("curl http://localhost:{}{}", port, example_path);
        rows.push_str(&format!(
            r#"<tr>
<td><span style="background:{};color:#fff;padding:2px 10px;border-radius:4px;font-weight:700;font-size:0.85em">{}</span></td>
<td style="font-family:monospace;font-weight:600"><a href="{}" style="color:#333;text-decoration:none">{}</a></td>
<td style="color:#666">{}()</td>
<td><code style="background:#f5f5f5;padding:2px 8px;border-radius:3px;font-size:0.85em">{}</code></td>
</tr>"#,
            method_color, route.method, example_path, route.path, route.fn_name, curl_cmd
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>Sunny Server</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:-apple-system,BlinkMacSystemFont,sans-serif;background:#fafafa;color:#333}}
header{{background:linear-gradient(135deg,#f7b731,#fc5c65);color:#fff;padding:3rem 2rem;text-align:center}}
header h1{{font-size:2.2rem;margin-bottom:0.5rem}}
header p{{opacity:0.9;font-size:1.1rem}}
main{{max-width:900px;margin:2rem auto;padding:0 1.5rem}}
table{{width:100%;border-collapse:collapse;background:#fff;border-radius:8px;overflow:hidden;box-shadow:0 1px 3px rgba(0,0,0,0.1)}}
th{{background:#f0f0f0;text-align:left;padding:12px 16px;font-size:0.85em;text-transform:uppercase;color:#666}}
td{{padding:12px 16px;border-top:1px solid #eee}}
tr:hover td{{background:#fafafa}}
.hint{{margin-top:2rem;padding:1.5rem;background:#fff9e6;border-left:4px solid #f7b731;border-radius:4px}}
.hint code{{background:#f0f0f0;padding:2px 6px;border-radius:3px}}
footer{{text-align:center;padding:2rem;color:#999;font-size:0.85rem}}
</style>
</head>
<body>
<header>
<h1>Sunny Server</h1>
<p>Listening on port {port}</p>
</header>
<main>
<h2 style="margin:1.5rem 0 1rem;color:#333">Routes</h2>
<table>
<tr><th>Method</th><th>Path</th><th>Handler</th><th>Try it</th></tr>
{rows}
</table>
<div class="hint">
<strong>Quick start</strong><br>
Pick any route above, or try: <code>curl http://localhost:{port}/health</code>
</div>
</main>
<footer>Powered by Sunny Lang</footer>
</body>
</html>"#,
        port = port,
        rows = rows,
    )
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

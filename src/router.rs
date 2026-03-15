use crate::ast::*;

/// HTTP 方法
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
        }
    }
}

/// 一條自動映射的路由
#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub method: HttpMethod,
    pub path: String,
    pub fn_name: String,
    pub params: Vec<Param>,
    pub body: Vec<Statement>,
}

/// Resource Action 後綴 → HTTP 方法 + 路徑模式
///
/// `fn productIndex()` → GET  /product
/// `fn productShow(id: Int)` → GET  /product/:id
/// `fn productStore(...)` → POST /product
/// `fn productUpdate(id: Int)` → PUT  /product/:id
/// `fn productRemove(id: Int)` → DELETE /product/:id
const SUFFIXES: &[(&str, HttpMethod, bool)] = &[
    ("Index", HttpMethod::Get, false),    // 無 :id
    ("Show", HttpMethod::Get, true),      // 有 :id
    ("Store", HttpMethod::Post, false),
    ("Update", HttpMethod::Put, true),
    ("Remove", HttpMethod::Delete, true),
];

/// 從函數名稱解析出 (resource, http_method, has_id)
fn parse_fn_name(name: &str) -> Option<(String, HttpMethod, bool)> {
    for (suffix, method, has_id) in SUFFIXES {
        if name.ends_with(suffix) {
            let resource = &name[..name.len() - suffix.len()];
            if resource.is_empty() {
                return None;
            }
            // camelCase resource → 小寫路徑
            let path_resource = resource[..1].to_lowercase() + &resource[1..];
            return Some((path_resource, method.clone(), *has_id));
        }
    }
    None
}

/// 從 AST 掃描所有頂層函數，自動產生路由表
pub fn build_routes(program: &Program) -> Vec<Route> {
    let mut routes = Vec::new();

    for stmt in &program.statements {
        if let Statement::Expression(Expression::Function {
            name,
            params,
            body,
            ..
        }) = stmt
        {
            if let Some((resource, method, has_id)) = parse_fn_name(name) {
                let path = if has_id {
                    format!("/{}/:id", resource)
                } else {
                    format!("/{}", resource)
                };
                routes.push(Route {
                    method,
                    path,
                    fn_name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                });
            }
        }
    }

    routes
}

/// 根據 HTTP method + path 查找對應路由，回傳 (Route, path_params)
/// path_params 例如 [("id", "42")] 從 /product/42 匹配 /product/:id
pub fn match_route<'a>(
    routes: &'a [Route],
    method: &str,
    request_path: &str,
) -> Option<(&'a Route, Vec<(String, String)>)> {
    let req_method = match method {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "DELETE" => HttpMethod::Delete,
        _ => return None,
    };

    for route in routes {
        if route.method != req_method {
            continue;
        }
        if let Some(params) = match_path(&route.path, request_path) {
            return Some((route, params));
        }
    }
    None
}

/// 路徑匹配: /product/:id 匹配 /product/42 → [("id", "42")]
fn match_path(pattern: &str, actual: &str) -> Option<Vec<(String, String)>> {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let actual_parts: Vec<&str> = actual.split('/').filter(|s| !s.is_empty()).collect();

    if pattern_parts.len() != actual_parts.len() {
        return None;
    }

    let mut params = Vec::new();
    for (p, a) in pattern_parts.iter().zip(actual_parts.iter()) {
        if p.starts_with(':') {
            params.push((p[1..].to_string(), a.to_string()));
        } else if p != a {
            return None;
        }
    }

    Some(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fn_name_index() {
        let (resource, method, has_id) = parse_fn_name("productIndex").unwrap();
        assert_eq!(resource, "product");
        assert_eq!(method, HttpMethod::Get);
        assert!(!has_id);
    }

    #[test]
    fn test_parse_fn_name_show() {
        let (resource, method, has_id) = parse_fn_name("userShow").unwrap();
        assert_eq!(resource, "user");
        assert_eq!(method, HttpMethod::Get);
        assert!(has_id);
    }

    #[test]
    fn test_parse_fn_name_store() {
        let (resource, method, _) = parse_fn_name("orderStore").unwrap();
        assert_eq!(resource, "order");
        assert_eq!(method, HttpMethod::Post);
    }

    #[test]
    fn test_parse_fn_name_update() {
        let (resource, method, has_id) = parse_fn_name("bookUpdate").unwrap();
        assert_eq!(resource, "book");
        assert_eq!(method, HttpMethod::Put);
        assert!(has_id);
    }

    #[test]
    fn test_parse_fn_name_remove() {
        let (resource, method, has_id) = parse_fn_name("taskRemove").unwrap();
        assert_eq!(resource, "task");
        assert_eq!(method, HttpMethod::Delete);
        assert!(has_id);
    }

    #[test]
    fn test_parse_fn_name_no_suffix() {
        assert!(parse_fn_name("helperFunc").is_none());
    }

    #[test]
    fn test_parse_fn_name_empty_resource() {
        assert!(parse_fn_name("Index").is_none());
    }

    #[test]
    fn test_match_path_static() {
        let params = match_path("/product", "/product").unwrap();
        assert!(params.is_empty());
    }

    #[test]
    fn test_match_path_with_param() {
        let params = match_path("/product/:id", "/product/42").unwrap();
        assert_eq!(params, vec![("id".to_string(), "42".to_string())]);
    }

    #[test]
    fn test_match_path_mismatch() {
        assert!(match_path("/product", "/user").is_none());
        assert!(match_path("/product/:id", "/product").is_none());
    }

    #[test]
    fn test_build_routes_from_ast() {
        let program = Program {
            statements: vec![
                Statement::Expression(Expression::Function {
                    name: "productIndex".to_string(),
                    params: vec![],
                    return_type: None,
                    body: vec![],
                }),
                Statement::Expression(Expression::Function {
                    name: "productShow".to_string(),
                    params: vec![Param {
                        name: "id".to_string(),
                        type_ann: TypeAnnotation::Int,
                    }],
                    return_type: None,
                    body: vec![],
                }),
                // 非 Resource Action 函數，不產生路由
                Statement::Expression(Expression::Function {
                    name: "helperCalc".to_string(),
                    params: vec![],
                    return_type: None,
                    body: vec![],
                }),
            ],
        };

        let routes = build_routes(&program);
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].method, HttpMethod::Get);
        assert_eq!(routes[0].path, "/product");
        assert_eq!(routes[0].fn_name, "productIndex");
        assert_eq!(routes[1].method, HttpMethod::Get);
        assert_eq!(routes[1].path, "/product/:id");
        assert_eq!(routes[1].fn_name, "productShow");
    }

    #[test]
    fn test_match_route() {
        let routes = vec![
            Route {
                method: HttpMethod::Get,
                path: "/product".to_string(),
                fn_name: "productIndex".to_string(),
                params: vec![],
                body: vec![],
            },
            Route {
                method: HttpMethod::Get,
                path: "/product/:id".to_string(),
                fn_name: "productShow".to_string(),
                params: vec![Param {
                    name: "id".to_string(),
                    type_ann: TypeAnnotation::Int,
                }],
                body: vec![],
            },
            Route {
                method: HttpMethod::Post,
                path: "/product".to_string(),
                fn_name: "productStore".to_string(),
                params: vec![],
                body: vec![],
            },
        ];

        // GET /product → productIndex
        let (route, params) = match_route(&routes, "GET", "/product").unwrap();
        assert_eq!(route.fn_name, "productIndex");
        assert!(params.is_empty());

        // GET /product/42 → productShow
        let (route, params) = match_route(&routes, "GET", "/product/42").unwrap();
        assert_eq!(route.fn_name, "productShow");
        assert_eq!(params[0].1, "42");

        // POST /product → productStore
        let (route, _) = match_route(&routes, "POST", "/product").unwrap();
        assert_eq!(route.fn_name, "productStore");

        // DELETE /product → None (no matching route)
        assert!(match_route(&routes, "DELETE", "/product").is_none());
    }
}

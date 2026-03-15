use crate::ast::*;
use crate::environment::{Environment, Value};

/// 求值結果：正常值或 output 回傳的值（需要冒泡）
#[derive(Debug, Clone, PartialEq)]
pub enum EvalResult {
    Val(Value),
    Output(Value), // out 語句產生，需向上冒泡
    Err(String),
}

/// Sunny 語言求值器：遞迴走訪 AST 進行求值
pub struct Evaluator {
    pub output_buffer: Vec<String>, // 攔截 print() 輸出，方便測試
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            output_buffer: Vec::new(),
        }
    }

    /// 執行整個程式
    pub fn eval_program(&mut self, program: &Program, env: &mut Environment) -> EvalResult {
        let mut result = EvalResult::Val(Value::Void);
        for stmt in &program.statements {
            result = self.eval_statement(stmt, env);
            match &result {
                EvalResult::Output(_) | EvalResult::Err(_) => return result,
                _ => {}
            }
        }
        result
    }

    // ── 語句求值 ──

    fn eval_statement(&mut self, stmt: &Statement, env: &mut Environment) -> EvalResult {
        match stmt {
            Statement::Lit { name, value } => {
                let val = match self.eval_expression(value, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                match env.define_lit(name, val) {
                    Ok(()) => EvalResult::Val(Value::Void),
                    Err(e) => EvalResult::Err(e),
                }
            }
            Statement::Glow { name, value } => {
                let val = match self.eval_expression(value, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                match env.define_glow(name, val) {
                    Ok(()) => EvalResult::Val(Value::Void),
                    Err(e) => EvalResult::Err(e),
                }
            }
            Statement::Assign { name, value } => {
                let val = match self.eval_expression(value, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                match env.assign(name, val) {
                    Ok(()) => EvalResult::Val(Value::Void),
                    Err(e) => EvalResult::Err(e),
                }
            }
            Statement::Out(expr) => {
                let val = match self.eval_expression(expr, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                EvalResult::Output(val)
            }
            Statement::Import { path } => self.eval_import(path, env),
            Statement::Expression(expr) => self.eval_expression(expr, env),
        }
    }

    /// 執行區塊（函數體、if 體等）
    pub fn eval_block(&mut self, stmts: &[Statement], env: &mut Environment) -> EvalResult {
        let mut result = EvalResult::Val(Value::Void);
        for stmt in stmts {
            result = self.eval_statement(stmt, env);
            match &result {
                EvalResult::Output(_) | EvalResult::Err(_) => return result,
                _ => {}
            }
        }
        result
    }

    // ── 表達式求值 ──

    fn eval_expression(&mut self, expr: &Expression, env: &mut Environment) -> EvalResult {
        match expr {
            // 字面量
            Expression::IntLiteral(v) => EvalResult::Val(Value::Int(*v)),
            Expression::FloatLiteral(v) => EvalResult::Val(Value::Float(*v)),
            Expression::StringLiteral(v) => {
                if v.contains('{') {
                    EvalResult::Val(Value::Str(self.interpolate_string(v, env)))
                } else {
                    EvalResult::Val(Value::Str(v.clone()))
                }
            }
            Expression::BoolLiteral(v) => EvalResult::Val(Value::Bool(*v)),

            // 識別符
            Expression::Ident(name) => match env.get(name) {
                Some(val) => EvalResult::Val(val),
                None => EvalResult::Err(format!("undefined variable: '{}'", name)),
            },

            // 前綴
            Expression::Prefix { operator, right } => {
                let right_val = match self.eval_expression(right, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                self.eval_prefix(operator, right_val)
            }

            // 中綴（and/or 短路求值）
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                let left_val = match self.eval_expression(left, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                // 短路求值: and/&& 左邊 false 就不求值右邊
                if matches!(operator, InfixOp::And) {
                    if let Value::Bool(false) = &left_val {
                        return EvalResult::Val(Value::Bool(false));
                    }
                }
                // 短路求值: or/|| 左邊 true 就不求值右邊
                if matches!(operator, InfixOp::Or) {
                    if let Value::Bool(true) = &left_val {
                        return EvalResult::Val(Value::Bool(true));
                    }
                }
                let right_val = match self.eval_expression(right, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                self.eval_infix(operator, left_val, right_val)
            }

            // 函數定義：具名綁定到環境，匿名直接回傳值
            Expression::Function {
                name, params, body, ..
            } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let func = Value::Function {
                    name: name.clone(),
                    params: param_names,
                    body: body.clone(),
                };
                if name.is_empty() {
                    // 匿名函數（閉包）：不綁定到環境，直接回傳
                    EvalResult::Val(func)
                } else {
                    match env.define_lit(name, func.clone()) {
                        Ok(()) => EvalResult::Val(func),
                        Err(e) => EvalResult::Err(e),
                    }
                }
            }

            // 函數呼叫
            Expression::Call { function, args } => self.eval_call(function, args, env),

            // if / else
            Expression::If {
                condition,
                consequence,
                alternative,
            } => {
                let cond = match self.eval_expression(condition, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                if is_truthy(&cond) {
                    self.eval_block(consequence, env)
                } else if let Some(alt) = alternative {
                    self.eval_block(alt, env)
                } else {
                    EvalResult::Val(Value::Void)
                }
            }

            // match
            Expression::Match { subject, arms } => {
                let subject_val = match self.eval_expression(subject, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                for arm in arms {
                    if type_matches(&subject_val, &arm.type_ann) {
                        let mut match_env = Environment::enclosed(env.clone());
                        if let Err(e) = match_env.define_lit(&arm.binding, subject_val) {
                            return EvalResult::Err(e);
                        }
                        return self.eval_block(&arm.body, &mut match_env);
                    }
                }
                EvalResult::Err(format!("no matching arm for value: {}", subject_val))
            }

            // for
            Expression::For {
                item,
                iterable,
                body,
            } => {
                let iter_val = match self.eval_expression(iterable, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                match iter_val {
                    Value::List(items) => {
                        let mut result = EvalResult::Val(Value::Void);
                        for val in items {
                            // 使用 glow 定義迴圈變數，讓每次迭代都能覆蓋
                            let mut loop_env = Environment::enclosed(env.clone());
                            let _ = loop_env.define_glow(item, val);
                            result = self.eval_block(body, &mut loop_env);
                            // 回寫 glow 變數的修改到外層 env
                            // (enclosed clone 了 outer，需要同步回來)
                            if let Some(outer) = loop_env.take_outer() {
                                *env = outer;
                            }
                            if matches!(&result, EvalResult::Output(_) | EvalResult::Err(_)) {
                                return result;
                            }
                        }
                        result
                    }
                    _ => EvalResult::Err("for requires a List to iterate over".to_string()),
                }
            }

            // while（上限 1,000,000 次迭代，防止無限迴圈凍結）
            Expression::While { condition, body } => {
                const MAX_ITERATIONS: usize = 1_000_000;
                let mut result = EvalResult::Val(Value::Void);
                let mut count = 0usize;
                loop {
                    let cond = match self.eval_expression(condition, env) {
                        EvalResult::Val(v) => v,
                        other => return other,
                    };
                    if !is_truthy(&cond) {
                        break;
                    }
                    count += 1;
                    if count > MAX_ITERATIONS {
                        return EvalResult::Err(format!(
                            "while loop exceeded {} iterations limit",
                            MAX_ITERATIONS
                        ));
                    }
                    result = self.eval_block(body, env);
                    if matches!(&result, EvalResult::Output(_) | EvalResult::Err(_)) {
                        return result;
                    }
                }
                result
            }

            // List 字面量
            Expression::ListLiteral(items) => {
                let mut values = Vec::new();
                for item in items {
                    match self.eval_expression(item, env) {
                        EvalResult::Val(v) => values.push(v),
                        other => return other,
                    }
                }
                EvalResult::Val(Value::List(values))
            }

            // Map 字面量
            Expression::MapLiteral(pairs) => {
                let mut entries = Vec::new();
                for (key_expr, val_expr) in pairs {
                    let key = match self.eval_expression(key_expr, env) {
                        EvalResult::Val(Value::Str(s)) => s,
                        EvalResult::Val(other) => {
                            return EvalResult::Err(format!(
                                "map key must be a String, got {}",
                                other
                            ))
                        }
                        other => return other,
                    };
                    let val = match self.eval_expression(val_expr, env) {
                        EvalResult::Val(v) => v,
                        other => return other,
                    };
                    entries.push((key, val));
                }
                EvalResult::Val(Value::Map(entries))
            }

            // 索引: list[0], map["key"]
            Expression::Index { left, index } => {
                let left_val = match self.eval_expression(left, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                let index_val = match self.eval_expression(index, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                self.eval_index(left_val, index_val)
            }

            // 屬性存取: shadow.message
            Expression::Dot { left, field } => {
                let left_val = match self.eval_expression(left, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                match (&left_val, field.as_str()) {
                    // Shadow 屬性
                    (Value::Shadow(msg), "message") => EvalResult::Val(Value::Str(msg.clone())),
                    // List 屬性
                    (Value::List(items), "length") => {
                        EvalResult::Val(Value::Int(items.len() as i64))
                    }
                    // String 屬性（字元數而非 byte 數）
                    (Value::Str(s), "length") => {
                        EvalResult::Val(Value::Int(s.chars().count() as i64))
                    }
                    // Map 屬性
                    (Value::Map(pairs), "length") => {
                        EvalResult::Val(Value::Int(pairs.len() as i64))
                    }
                    _ => EvalResult::Err(format!(
                        "cannot access '.{}' on {}",
                        field, left_val
                    )),
                }
            }

            // Shadow 建構
            Expression::ShadowLiteral(inner) => {
                let val = match self.eval_expression(inner, env) {
                    EvalResult::Val(v) => v,
                    other => return other,
                };
                match val {
                    Value::Str(msg) => EvalResult::Val(Value::Shadow(msg)),
                    _ => EvalResult::Err("Shadow() expects a String argument".to_string()),
                }
            }

            // Range: 0..10 → List [0, 1, 2, ..., 9]
            Expression::Range { start, end } => {
                let start_val = match self.eval_expression(start, env) {
                    EvalResult::Val(Value::Int(n)) => n,
                    EvalResult::Val(other) => {
                        return EvalResult::Err(format!("range start must be Int, got {}", other))
                    }
                    other => return other,
                };
                let end_val = match self.eval_expression(end, env) {
                    EvalResult::Val(Value::Int(n)) => n,
                    EvalResult::Val(other) => {
                        return EvalResult::Err(format!("range end must be Int, got {}", other))
                    }
                    other => return other,
                };
                const MAX_RANGE: i64 = 1_000_000;
                let size = end_val - start_val;
                if size < 0 {
                    EvalResult::Val(Value::List(vec![]))
                } else if size > MAX_RANGE {
                    EvalResult::Err(format!(
                        "range size {} exceeds limit {}",
                        size, MAX_RANGE
                    ))
                } else {
                    let items: Vec<Value> = (start_val..end_val).map(Value::Int).collect();
                    EvalResult::Val(Value::List(items))
                }
            }

            // ray 併發：在新執行緒中執行，回傳 RayHandle
            Expression::Ray { body } => {
                use crate::environment::RayResult;
                use std::sync::{Arc, Mutex};

                let ray_env = Environment::enclosed(env.clone());
                let body_clone = body.clone();
                let result_slot: Arc<Mutex<Option<Result<Value, String>>>> =
                    Arc::new(Mutex::new(None));
                let result_writer = result_slot.clone();

                let handle = std::thread::spawn(move || {
                    let mut eval = Evaluator::new();
                    let mut env = ray_env;
                    let res = eval.eval_block(&body_clone, &mut env);
                    let outcome = match res {
                        EvalResult::Val(v) | EvalResult::Output(v) => Ok(v),
                        EvalResult::Err(msg) => Err(msg),
                    };
                    *result_writer.lock().unwrap() = Some(outcome);
                });

                EvalResult::Val(Value::RayHandle(RayResult {
                    value: result_slot,
                    handle: Arc::new(Mutex::new(Some(handle))),
                }))
            }
        }
    }

    // ── 運算子求值 ──

    fn eval_prefix(&self, op: &PrefixOp, right: Value) -> EvalResult {
        match (op, &right) {
            (PrefixOp::Negate, Value::Int(v)) => EvalResult::Val(Value::Int(-v)),
            (PrefixOp::Negate, Value::Float(v)) => EvalResult::Val(Value::Float(-v)),
            (PrefixOp::Bang | PrefixOp::Not, Value::Bool(v)) => {
                EvalResult::Val(Value::Bool(!v))
            }
            _ => EvalResult::Err(format!("invalid prefix operation: {:?} on {}", op, right)),
        }
    }

    fn eval_infix(&self, op: &InfixOp, left: Value, right: Value) -> EvalResult {
        match (op, &left, &right) {
            // Int 算術
            (InfixOp::Add, Value::Int(a), Value::Int(b)) => EvalResult::Val(Value::Int(a + b)),
            (InfixOp::Sub, Value::Int(a), Value::Int(b)) => EvalResult::Val(Value::Int(a - b)),
            (InfixOp::Mul, Value::Int(a), Value::Int(b)) => EvalResult::Val(Value::Int(a * b)),
            (InfixOp::Div, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    EvalResult::Val(Value::Shadow("division by zero".to_string()))
                } else {
                    EvalResult::Val(Value::Int(a / b))
                }
            }
            (InfixOp::Mod, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    EvalResult::Val(Value::Shadow("modulo by zero".to_string()))
                } else {
                    EvalResult::Val(Value::Int(a % b))
                }
            }

            // Float 算術
            (InfixOp::Add, Value::Float(a), Value::Float(b)) => {
                EvalResult::Val(Value::Float(a + b))
            }
            (InfixOp::Sub, Value::Float(a), Value::Float(b)) => {
                EvalResult::Val(Value::Float(a - b))
            }
            (InfixOp::Mul, Value::Float(a), Value::Float(b)) => {
                EvalResult::Val(Value::Float(a * b))
            }
            (InfixOp::Div, Value::Float(_), Value::Float(b)) if *b == 0.0 => {
                EvalResult::Val(Value::Shadow("division by zero".to_string()))
            }
            (InfixOp::Div, Value::Float(a), Value::Float(b)) => {
                EvalResult::Val(Value::Float(a / b))
            }

            // Int + Float 混合運算
            (InfixOp::Add, Value::Int(a), Value::Float(b)) => {
                EvalResult::Val(Value::Float(*a as f64 + b))
            }
            (InfixOp::Add, Value::Float(a), Value::Int(b)) => {
                EvalResult::Val(Value::Float(a + *b as f64))
            }
            (InfixOp::Sub, Value::Int(a), Value::Float(b)) => {
                EvalResult::Val(Value::Float(*a as f64 - b))
            }
            (InfixOp::Sub, Value::Float(a), Value::Int(b)) => {
                EvalResult::Val(Value::Float(a - *b as f64))
            }
            (InfixOp::Mul, Value::Int(a), Value::Float(b)) => {
                EvalResult::Val(Value::Float(*a as f64 * b))
            }
            (InfixOp::Mul, Value::Float(a), Value::Int(b)) => {
                EvalResult::Val(Value::Float(a * *b as f64))
            }
            (InfixOp::Div, Value::Int(_), Value::Float(b)) if *b == 0.0 => {
                EvalResult::Val(Value::Shadow("division by zero".to_string()))
            }
            (InfixOp::Div, Value::Int(a), Value::Float(b)) => {
                EvalResult::Val(Value::Float(*a as f64 / b))
            }
            (InfixOp::Div, Value::Float(_), Value::Int(b)) if *b == 0 => {
                EvalResult::Val(Value::Shadow("division by zero".to_string()))
            }
            (InfixOp::Div, Value::Float(a), Value::Int(b)) => {
                EvalResult::Val(Value::Float(a / *b as f64))
            }

            // 字串串接
            (InfixOp::Add, Value::Str(a), Value::Str(b)) => {
                EvalResult::Val(Value::Str(format!("{}{}", a, b)))
            }

            // Int 比較
            (InfixOp::Eq, Value::Int(a), Value::Int(b)) => {
                EvalResult::Val(Value::Bool(a == b))
            }
            (InfixOp::NotEq, Value::Int(a), Value::Int(b)) => {
                EvalResult::Val(Value::Bool(a != b))
            }
            (InfixOp::Lt, Value::Int(a), Value::Int(b)) => {
                EvalResult::Val(Value::Bool(a < b))
            }
            (InfixOp::Gt, Value::Int(a), Value::Int(b)) => {
                EvalResult::Val(Value::Bool(a > b))
            }
            (InfixOp::LtEq, Value::Int(a), Value::Int(b)) => {
                EvalResult::Val(Value::Bool(a <= b))
            }
            (InfixOp::GtEq, Value::Int(a), Value::Int(b)) => {
                EvalResult::Val(Value::Bool(a >= b))
            }

            // 字串比較
            (InfixOp::Eq, Value::Str(a), Value::Str(b)) => {
                EvalResult::Val(Value::Bool(a == b))
            }
            (InfixOp::NotEq, Value::Str(a), Value::Str(b)) => {
                EvalResult::Val(Value::Bool(a != b))
            }

            // Bool 比較
            (InfixOp::Eq, Value::Bool(a), Value::Bool(b)) => {
                EvalResult::Val(Value::Bool(a == b))
            }
            (InfixOp::NotEq, Value::Bool(a), Value::Bool(b)) => {
                EvalResult::Val(Value::Bool(a != b))
            }

            // 邏輯運算
            (InfixOp::And, Value::Bool(a), Value::Bool(b)) => {
                EvalResult::Val(Value::Bool(*a && *b))
            }
            (InfixOp::Or, Value::Bool(a), Value::Bool(b)) => {
                EvalResult::Val(Value::Bool(*a || *b))
            }

            _ => EvalResult::Err(format!(
                "unsupported operation: {} {:?} {}",
                left, op, right
            )),
        }
    }

    fn eval_index(&self, left: Value, index: Value) -> EvalResult {
        match (&left, &index) {
            (Value::List(items), Value::Int(i)) => {
                if *i < 0 {
                    EvalResult::Val(Value::Shadow(format!(
                        "negative index {} on list (length {})",
                        i,
                        items.len()
                    )))
                } else {
                    let idx = *i as usize;
                    if idx < items.len() {
                        EvalResult::Val(items[idx].clone())
                    } else {
                        EvalResult::Val(Value::Shadow(format!(
                            "index {} out of bounds (length {})",
                            i,
                            items.len()
                        )))
                    }
                }
            }
            (Value::Map(pairs), Value::Str(key)) => {
                for (k, v) in pairs {
                    if k == key {
                        return EvalResult::Val(v.clone());
                    }
                }
                EvalResult::Val(Value::Shadow(format!("key '{}' not found", key)))
            }
            _ => EvalResult::Err(format!("cannot index {} with {}", left, index)),
        }
    }

    // ── 函數呼叫 ──

    fn eval_call(
        &mut self,
        function: &Expression,
        args: &[Expression],
        env: &mut Environment,
    ) -> EvalResult {
        // 內建函數
        if let Expression::Ident(name) = function {
            match name.as_str() {
                "print" => return self.builtin_print(args, env),
                "await" => return self.builtin_await(args, env),
                "read_file" => return self.builtin_read_file(args, env),
                "render_md" => return self.builtin_render_md(args, env),
                "render_template" => return self.builtin_render_template(args, env),
                "write_file" => return self.builtin_write_file(args, env),
                "len" => return self.builtin_len(args, env),
                "type_of" => return self.builtin_type_of(args, env),
                "to_int" => return self.builtin_to_int(args, env),
                "to_float" => return self.builtin_to_float(args, env),
                "to_string" => return self.builtin_to_string(args, env),
                "json_encode" => return self.builtin_json_encode(args, env),
                "time_now" => return self.builtin_time_now(args, env),
                _ => {}
            }
        }

        // 方法呼叫: obj.method(args)
        if let Expression::Dot { left, field } = function {
            let receiver = match self.eval_expression(left, env) {
                EvalResult::Val(v) => v,
                other => return other,
            };
            let mut eval_args = Vec::new();
            for arg in args {
                match self.eval_expression(arg, env) {
                    EvalResult::Val(v) => eval_args.push(v),
                    other => return other,
                }
            }
            return self.eval_method(receiver, field, eval_args);
        }

        let func_val = match self.eval_expression(function, env) {
            EvalResult::Val(v) => v,
            other => return other,
        };

        match func_val {
            Value::Function {
                params, body, ..
            } => {
                if args.len() != params.len() {
                    return EvalResult::Err(format!(
                        "expected {} arguments, got {}",
                        params.len(),
                        args.len()
                    ));
                }
                let mut call_env = Environment::enclosed(env.clone());
                for (param_name, arg_expr) in params.iter().zip(args.iter()) {
                    let val = match self.eval_expression(arg_expr, env) {
                        EvalResult::Val(v) => v,
                        other => return other,
                    };
                    call_env.define_lit(param_name, val).unwrap();
                }
                match self.eval_block(&body, &mut call_env) {
                    EvalResult::Output(v) => EvalResult::Val(v), // out 冒泡後解包為正常值
                    other => other,
                }
            }
            _ => EvalResult::Err(format!("{} is not a function", func_val)),
        }
    }

    // ── 方法呼叫 ──

    fn eval_method(&self, receiver: Value, method: &str, args: Vec<Value>) -> EvalResult {
        match (&receiver, method) {
            // ── List 方法 ──
            (Value::List(items), "push") => {
                if args.len() != 1 {
                    return EvalResult::Err("List.push() expects 1 argument".to_string());
                }
                let mut new_items = items.clone();
                new_items.push(args.into_iter().next().unwrap());
                EvalResult::Val(Value::List(new_items))
            }
            (Value::List(items), "pop") => {
                if items.is_empty() {
                    return EvalResult::Val(Value::Shadow("pop on empty list".to_string()));
                }
                let mut new_items = items.clone();
                new_items.pop();
                EvalResult::Val(Value::List(new_items))
            }
            (Value::List(items), "first") => {
                match items.first() {
                    Some(v) => EvalResult::Val(v.clone()),
                    None => EvalResult::Val(Value::Shadow("first on empty list".to_string())),
                }
            }
            (Value::List(items), "last") => {
                match items.last() {
                    Some(v) => EvalResult::Val(v.clone()),
                    None => EvalResult::Val(Value::Shadow("last on empty list".to_string())),
                }
            }
            (Value::List(items), "contains") => {
                if args.len() != 1 {
                    return EvalResult::Err("List.contains() expects 1 argument".to_string());
                }
                let target = &args[0];
                EvalResult::Val(Value::Bool(items.contains(target)))
            }
            (Value::List(items), "reverse") => {
                let mut new_items = items.clone();
                new_items.reverse();
                EvalResult::Val(Value::List(new_items))
            }
            (Value::List(items), "join") => {
                if args.len() != 1 {
                    return EvalResult::Err("List.join() expects 1 argument (separator)".to_string());
                }
                match &args[0] {
                    Value::Str(sep) => {
                        let parts: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
                        EvalResult::Val(Value::Str(parts.join(sep)))
                    }
                    _ => EvalResult::Err("List.join() separator must be a String".to_string()),
                }
            }

            // ── Map 方法 ──
            (Value::Map(pairs), "keys") => {
                let keys: Vec<Value> = pairs.iter().map(|(k, _)| Value::Str(k.clone())).collect();
                EvalResult::Val(Value::List(keys))
            }
            (Value::Map(pairs), "values") => {
                let vals: Vec<Value> = pairs.iter().map(|(_, v)| v.clone()).collect();
                EvalResult::Val(Value::List(vals))
            }
            (Value::Map(pairs), "has") => {
                if args.len() != 1 {
                    return EvalResult::Err("Map.has() expects 1 argument".to_string());
                }
                match &args[0] {
                    Value::Str(key) => {
                        let found = pairs.iter().any(|(k, _)| k == key);
                        EvalResult::Val(Value::Bool(found))
                    }
                    _ => EvalResult::Err("Map.has() key must be a String".to_string()),
                }
            }
            (Value::Map(pairs), "length") => {
                EvalResult::Val(Value::Int(pairs.len() as i64))
            }
            (Value::Map(pairs), "remove") => {
                if args.len() != 1 {
                    return EvalResult::Err("Map.remove() expects 1 argument".to_string());
                }
                match &args[0] {
                    Value::Str(key) => {
                        let new_pairs: Vec<(String, Value)> = pairs
                            .iter()
                            .filter(|(k, _)| k != key)
                            .cloned()
                            .collect();
                        EvalResult::Val(Value::Map(new_pairs))
                    }
                    _ => EvalResult::Err("Map.remove() key must be a String".to_string()),
                }
            }

            // ── String 方法 ──
            (Value::Str(s), "length") => {
                EvalResult::Val(Value::Int(s.chars().count() as i64))
            }
            (Value::Str(s), "trim") => {
                EvalResult::Val(Value::Str(s.trim().to_string()))
            }
            (Value::Str(s), "upper") => {
                EvalResult::Val(Value::Str(s.to_uppercase()))
            }
            (Value::Str(s), "lower") => {
                EvalResult::Val(Value::Str(s.to_lowercase()))
            }
            (Value::Str(s), "contains") => {
                if args.len() != 1 {
                    return EvalResult::Err("String.contains() expects 1 argument".to_string());
                }
                match &args[0] {
                    Value::Str(sub) => EvalResult::Val(Value::Bool(s.contains(sub.as_str()))),
                    _ => EvalResult::Err("String.contains() argument must be a String".to_string()),
                }
            }
            (Value::Str(s), "starts_with") => {
                if args.len() != 1 {
                    return EvalResult::Err("String.starts_with() expects 1 argument".to_string());
                }
                match &args[0] {
                    Value::Str(prefix) => EvalResult::Val(Value::Bool(s.starts_with(prefix.as_str()))),
                    _ => EvalResult::Err("String.starts_with() argument must be a String".to_string()),
                }
            }
            (Value::Str(s), "ends_with") => {
                if args.len() != 1 {
                    return EvalResult::Err("String.ends_with() expects 1 argument".to_string());
                }
                match &args[0] {
                    Value::Str(suffix) => EvalResult::Val(Value::Bool(s.ends_with(suffix.as_str()))),
                    _ => EvalResult::Err("String.ends_with() argument must be a String".to_string()),
                }
            }
            (Value::Str(s), "split") => {
                if args.len() != 1 {
                    return EvalResult::Err("String.split() expects 1 argument".to_string());
                }
                match &args[0] {
                    Value::Str(sep) => {
                        let parts: Vec<Value> = s.split(sep.as_str())
                            .map(|p| Value::Str(p.to_string()))
                            .collect();
                        EvalResult::Val(Value::List(parts))
                    }
                    _ => EvalResult::Err("String.split() separator must be a String".to_string()),
                }
            }

            // ── Shadow 方法 ──
            (Value::Shadow(msg), "wrap") => {
                if args.len() != 1 {
                    return EvalResult::Err("Shadow.wrap() expects 1 argument".to_string());
                }
                match &args[0] {
                    Value::Str(prefix) => {
                        EvalResult::Val(Value::Shadow(format!("{}: {}", prefix, msg)))
                    }
                    _ => EvalResult::Err("Shadow.wrap() argument must be a String".to_string()),
                }
            }
            (Value::Shadow(_), "unwrap_or") => {
                if args.len() != 1 {
                    return EvalResult::Err("Shadow.unwrap_or() expects 1 argument".to_string());
                }
                EvalResult::Val(args.into_iter().next().unwrap())
            }

            _ => EvalResult::Err(format!(
                "no method '.{}()' on {}",
                method, receiver
            )),
        }
    }

    fn builtin_print(&mut self, args: &[Expression], env: &mut Environment) -> EvalResult {
        let mut parts = Vec::new();
        for arg in args {
            match self.eval_expression(arg, env) {
                EvalResult::Val(v) => parts.push(format!("{}", v)),
                other => return other,
            }
        }
        let output = parts.join(" ");
        self.output_buffer.push(output);
        EvalResult::Val(Value::Void)
    }

    /// `await(ray_handle)` — 等待 ray 執行緒完成並取得結果
    fn builtin_await(&mut self, args: &[Expression], env: &mut Environment) -> EvalResult {
        if args.len() != 1 {
            return EvalResult::Err("await() expects 1 argument".to_string());
        }
        let val = match self.eval_expression(&args[0], env) {
            EvalResult::Val(v) => v,
            other => return other,
        };
        match val {
            Value::RayHandle(ray_result) => {
                // 等待執行緒結束
                if let Some(handle) = ray_result.handle.lock().unwrap().take() {
                    let _ = handle.join();
                }
                // 取得結果
                match ray_result.value.lock().unwrap().take() {
                    Some(Ok(v)) => EvalResult::Val(v),
                    Some(Err(msg)) => EvalResult::Err(msg),
                    None => EvalResult::Val(Value::Void),
                }
            }
            _ => EvalResult::Err("await() expects a ray handle".to_string()),
        }
    }

    /// `read_file(path)` — 讀取檔案內容，回傳 String 或 Shadow
    fn builtin_read_file(&mut self, args: &[Expression], env: &mut Environment) -> EvalResult {
        if args.len() != 1 {
            return EvalResult::Err("read_file() expects 1 argument".to_string());
        }
        let val = match self.eval_expression(&args[0], env) {
            EvalResult::Val(v) => v,
            other => return other,
        };
        match val {
            Value::Str(path) => match std::fs::read_to_string(&path) {
                Ok(content) => EvalResult::Val(Value::Str(content)),
                Err(e) => EvalResult::Val(Value::Shadow(format!(
                    "cannot read '{}': {}",
                    path, e
                ))),
            },
            _ => EvalResult::Err("read_file() expects a String path".to_string()),
        }
    }

    /// `render_md(markdown_string)` — 將 Markdown 轉為 HTML
    fn builtin_render_md(&mut self, args: &[Expression], env: &mut Environment) -> EvalResult {
        if args.len() != 1 {
            return EvalResult::Err("render_md() expects 1 argument".to_string());
        }
        let val = match self.eval_expression(&args[0], env) {
            EvalResult::Val(v) => v,
            other => return other,
        };
        match val {
            Value::Str(md) => {
                let html = crate::markdown::render(&md);
                EvalResult::Val(Value::Str(html))
            }
            _ => EvalResult::Err("render_md() expects a String".to_string()),
        }
    }

    /// `render_template(template, vars_map)` — 將 {{ key }} 替換為 map 中的值
    fn builtin_render_template(
        &mut self,
        args: &[Expression],
        env: &mut Environment,
    ) -> EvalResult {
        if args.len() != 2 {
            return EvalResult::Err("render_template() expects 2 arguments".to_string());
        }
        let tmpl = match self.eval_expression(&args[0], env) {
            EvalResult::Val(Value::Str(s)) => s,
            EvalResult::Val(_) => {
                return EvalResult::Err("render_template() first arg must be a String".to_string())
            }
            other => return other,
        };
        let vars = match self.eval_expression(&args[1], env) {
            EvalResult::Val(Value::Map(pairs)) => pairs,
            EvalResult::Val(_) => {
                return EvalResult::Err("render_template() second arg must be a Map".to_string())
            }
            other => return other,
        };
        let html = crate::template::render(&tmpl, &vars);
        EvalResult::Val(Value::Str(html))
    }
    /// `import "path.sunny"` — 讀取並執行外部模組，將定義匯入當前環境
    fn eval_import(&mut self, path: &str, env: &mut Environment) -> EvalResult {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                return EvalResult::Err(format!("cannot import '{}': {}", path, e));
            }
        };
        let mut parser = crate::parser::Parser::new(&source);
        let program = parser.parse();
        if !parser.errors.is_empty() {
            return EvalResult::Err(format!(
                "parse error in '{}': {}",
                path,
                parser.errors.join(", ")
            ));
        }
        let result = self.eval_program(&program, env);
        match result {
            EvalResult::Err(msg) => EvalResult::Err(format!("in '{}': {}", path, msg)),
            _ => EvalResult::Val(Value::Void),
        }
    }

    /// `write_file(path, content)` — 寫入檔案
    fn builtin_write_file(&mut self, args: &[Expression], env: &mut Environment) -> EvalResult {
        if args.len() != 2 {
            return EvalResult::Err("write_file() expects 2 arguments".to_string());
        }
        let path = match self.eval_expression(&args[0], env) {
            EvalResult::Val(Value::Str(s)) => s,
            EvalResult::Val(_) => return EvalResult::Err("write_file() path must be a String".to_string()),
            other => return other,
        };
        let content = match self.eval_expression(&args[1], env) {
            EvalResult::Val(Value::Str(s)) => s,
            EvalResult::Val(_) => return EvalResult::Err("write_file() content must be a String".to_string()),
            other => return other,
        };
        // 自動建立父目錄
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::write(&path, &content) {
            Ok(()) => EvalResult::Val(Value::Bool(true)),
            Err(e) => EvalResult::Val(Value::Shadow(format!("cannot write '{}': {}", path, e))),
        }
    }

    /// `len(x)` — 統一取長度
    fn builtin_len(&mut self, args: &[Expression], env: &mut Environment) -> EvalResult {
        if args.len() != 1 {
            return EvalResult::Err("len() expects 1 argument".to_string());
        }
        let val = match self.eval_expression(&args[0], env) {
            EvalResult::Val(v) => v,
            other => return other,
        };
        match &val {
            Value::Str(s) => EvalResult::Val(Value::Int(s.chars().count() as i64)),
            Value::List(items) => EvalResult::Val(Value::Int(items.len() as i64)),
            Value::Map(pairs) => EvalResult::Val(Value::Int(pairs.len() as i64)),
            _ => EvalResult::Err(format!("len() not supported for {}", val)),
        }
    }

    /// `type_of(x)` — 回傳型別名稱
    fn builtin_type_of(&mut self, args: &[Expression], env: &mut Environment) -> EvalResult {
        if args.len() != 1 {
            return EvalResult::Err("type_of() expects 1 argument".to_string());
        }
        let val = match self.eval_expression(&args[0], env) {
            EvalResult::Val(v) => v,
            other => return other,
        };
        let name = match &val {
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Str(_) => "String",
            Value::Bool(_) => "Bool",
            Value::List(_) => "List",
            Value::Map(_) => "Map",
            Value::Shadow(_) => "Shadow",
            Value::Function { .. } => "Function",
            Value::RayHandle(_) => "Ray",
            Value::Void => "Void",
        };
        EvalResult::Val(Value::Str(name.to_string()))
    }

    /// `to_int(x)` — 轉換為 Int
    fn builtin_to_int(&mut self, args: &[Expression], env: &mut Environment) -> EvalResult {
        if args.len() != 1 {
            return EvalResult::Err("to_int() expects 1 argument".to_string());
        }
        let val = match self.eval_expression(&args[0], env) {
            EvalResult::Val(v) => v,
            other => return other,
        };
        match &val {
            Value::Int(_) => EvalResult::Val(val),
            Value::Float(f) => EvalResult::Val(Value::Int(*f as i64)),
            Value::Str(s) => match s.parse::<i64>() {
                Ok(n) => EvalResult::Val(Value::Int(n)),
                Err(_) => EvalResult::Val(Value::Shadow(format!("cannot convert '{}' to Int", s))),
            },
            Value::Bool(b) => EvalResult::Val(Value::Int(if *b { 1 } else { 0 })),
            _ => EvalResult::Val(Value::Shadow(format!("cannot convert {} to Int", val))),
        }
    }

    /// `to_float(x)` — 轉換為 Float
    fn builtin_to_float(&mut self, args: &[Expression], env: &mut Environment) -> EvalResult {
        if args.len() != 1 {
            return EvalResult::Err("to_float() expects 1 argument".to_string());
        }
        let val = match self.eval_expression(&args[0], env) {
            EvalResult::Val(v) => v,
            other => return other,
        };
        match &val {
            Value::Float(_) => EvalResult::Val(val),
            Value::Int(n) => EvalResult::Val(Value::Float(*n as f64)),
            Value::Str(s) => match s.parse::<f64>() {
                Ok(f) => EvalResult::Val(Value::Float(f)),
                Err(_) => EvalResult::Val(Value::Shadow(format!("cannot convert '{}' to Float", s))),
            },
            _ => EvalResult::Val(Value::Shadow(format!("cannot convert {} to Float", val))),
        }
    }

    /// `to_string(x)` — 轉換為 String
    fn builtin_to_string(&mut self, args: &[Expression], env: &mut Environment) -> EvalResult {
        if args.len() != 1 {
            return EvalResult::Err("to_string() expects 1 argument".to_string());
        }
        let val = match self.eval_expression(&args[0], env) {
            EvalResult::Val(v) => v,
            other => return other,
        };
        EvalResult::Val(Value::Str(format!("{}", val)))
    }

    /// `json_encode(value)` — 值轉為 JSON 字串
    fn builtin_json_encode(&mut self, args: &[Expression], env: &mut Environment) -> EvalResult {
        if args.len() != 1 {
            return EvalResult::Err("json_encode() expects 1 argument".to_string());
        }
        let val = match self.eval_expression(&args[0], env) {
            EvalResult::Val(v) => v,
            other => return other,
        };
        EvalResult::Val(Value::Str(value_to_json(&val)))
    }

    /// `time_now()` — 當前 Unix timestamp
    fn builtin_time_now(&mut self, args: &[Expression], _env: &mut Environment) -> EvalResult {
        if !args.is_empty() {
            return EvalResult::Err("time_now() takes no arguments".to_string());
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        EvalResult::Val(Value::Int(now as i64))
    }

    /// 字串內插: "hello {name}!" → 替換 {identifier} 為環境中的值
    /// 只有 {合法識別符} 會被替換，{" 開頭的不會觸發（避免和 JSON 衝突）
    fn interpolate_string(&self, template: &str, env: &Environment) -> String {
        let mut result = String::new();
        let mut chars = template.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // 偷看下一個字元，只有字母或底線開頭才視為內插
                if let Some(&next) = chars.peek() {
                    if next.is_ascii_alphabetic() || next == '_' {
                        let mut var_name = String::new();
                        let mut found_close = false;
                        while let Some(&c) = chars.peek() {
                            if c == '}' {
                                chars.next();
                                found_close = true;
                                break;
                            }
                            if c.is_ascii_alphanumeric() || c == '_' {
                                var_name.push(c);
                                chars.next();
                            } else {
                                break; // 非合法識別符字元，不做替換
                            }
                        }
                        if found_close {
                            if let Some(val) = env.get(&var_name) {
                                result.push_str(&format!("{}", val));
                            } else {
                                result.push('{');
                                result.push_str(&var_name);
                                result.push('}');
                            }
                        } else {
                            result.push('{');
                            result.push_str(&var_name);
                        }
                        continue;
                    }
                }
                result.push('{');
            } else {
                result.push(ch);
            }
        }
        result
    }
}

/// Value → JSON 字串
fn value_to_json(val: &Value) -> String {
    match val {
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Bool(b) => b.to_string(),
        Value::List(items) => {
            let parts: Vec<String> = items.iter().map(value_to_json).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Map(pairs) => {
            let parts: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, value_to_json(v)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        Value::Shadow(msg) => format!("{{\"shadow\": \"{}\"}}", msg),
        Value::Void => "null".to_string(),
        Value::Function { name, .. } => format!("\"<fn {}>\"", name),
        Value::RayHandle(_) => "\"<ray>\"".to_string(),
    }
}

// ── 輔助函數 ──

fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Bool(b) => *b,
        Value::Int(n) => *n != 0,
        Value::Str(s) => !s.is_empty(),
        Value::Void => false,
        Value::Shadow(_) => false,
        _ => true,
    }
}

fn type_matches(val: &Value, ann: &TypeAnnotation) -> bool {
    match (val, ann) {
        (Value::Int(_), TypeAnnotation::Int) => true,
        (Value::Float(_), TypeAnnotation::Float) => true,
        (Value::Str(_), TypeAnnotation::Str) => true,
        (Value::Bool(_), TypeAnnotation::Bool) => true,
        (Value::List(_), TypeAnnotation::List) => true,
        (Value::Map(_), TypeAnnotation::Map) => true,
        (Value::Shadow(_), TypeAnnotation::Shadow) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(input: &str) -> (EvalResult, Vec<String>) {
        let mut parser = crate::parser::Parser::new(input);
        let program = parser.parse();
        if !parser.errors.is_empty() {
            return (
                EvalResult::Err(format!("parse errors: {:?}", parser.errors)),
                vec![],
            );
        }
        let mut evaluator = Evaluator::new();
        let mut env = Environment::new();
        let result = evaluator.eval_program(&program, &mut env);
        (result, evaluator.output_buffer)
    }

    fn eval(input: &str) -> EvalResult {
        run(input).0
    }

    fn eval_prints(input: &str) -> Vec<String> {
        run(input).1
    }

    // ── 字面量 ──

    #[test]
    fn test_int_literal() {
        assert_eq!(eval("42"), EvalResult::Val(Value::Int(42)));
    }

    #[test]
    fn test_float_literal() {
        assert_eq!(eval("3.14"), EvalResult::Val(Value::Float(3.14)));
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(
            eval(r#""hello""#),
            EvalResult::Val(Value::Str("hello".to_string()))
        );
    }

    #[test]
    fn test_bool_literal() {
        assert_eq!(eval("true"), EvalResult::Val(Value::Bool(true)));
    }

    // ── 算術 ──

    #[test]
    fn test_arithmetic() {
        assert_eq!(eval("2 + 3"), EvalResult::Val(Value::Int(5)));
        assert_eq!(eval("10 - 4"), EvalResult::Val(Value::Int(6)));
        assert_eq!(eval("3 * 7"), EvalResult::Val(Value::Int(21)));
        assert_eq!(eval("15 / 3"), EvalResult::Val(Value::Int(5)));
        assert_eq!(eval("10 % 3"), EvalResult::Val(Value::Int(1)));
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(eval("2 + 3 * 4"), EvalResult::Val(Value::Int(14)));
        assert_eq!(eval("(2 + 3) * 4"), EvalResult::Val(Value::Int(20)));
    }

    #[test]
    fn test_division_by_zero_returns_shadow() {
        assert_eq!(
            eval("10 / 0"),
            EvalResult::Val(Value::Shadow("division by zero".to_string()))
        );
    }

    #[test]
    fn test_string_concat() {
        assert_eq!(
            eval(r#""hello" + " world""#),
            EvalResult::Val(Value::Str("hello world".to_string()))
        );
    }

    #[test]
    fn test_mixed_int_float() {
        assert_eq!(eval("1 + 2.5"), EvalResult::Val(Value::Float(3.5)));
        assert_eq!(eval("2.5 * 2"), EvalResult::Val(Value::Float(5.0)));
    }

    // ── 比較與邏輯 ──

    #[test]
    fn test_comparison() {
        assert_eq!(eval("5 > 3"), EvalResult::Val(Value::Bool(true)));
        assert_eq!(eval("5 < 3"), EvalResult::Val(Value::Bool(false)));
        assert_eq!(eval("5 == 5"), EvalResult::Val(Value::Bool(true)));
        assert_eq!(eval("5 != 3"), EvalResult::Val(Value::Bool(true)));
        assert_eq!(eval("3 <= 3"), EvalResult::Val(Value::Bool(true)));
        assert_eq!(eval("3 >= 5"), EvalResult::Val(Value::Bool(false)));
    }

    #[test]
    fn test_logical() {
        assert_eq!(eval("true and false"), EvalResult::Val(Value::Bool(false)));
        assert_eq!(eval("true or false"), EvalResult::Val(Value::Bool(true)));
        assert_eq!(eval("not true"), EvalResult::Val(Value::Bool(false)));
        assert_eq!(eval("!false"), EvalResult::Val(Value::Bool(true)));
    }

    // ── 前綴 ──

    #[test]
    fn test_prefix_negate() {
        assert_eq!(eval("-42"), EvalResult::Val(Value::Int(-42)));
        assert_eq!(eval("-3.14"), EvalResult::Val(Value::Float(-3.14)));
    }

    // ── 變數綁定 ──

    #[test]
    fn test_lit_binding() {
        assert_eq!(
            eval("lit x = 42\nx"),
            EvalResult::Val(Value::Int(42))
        );
    }

    #[test]
    fn test_glow_reassign() {
        assert_eq!(
            eval("glow x = 1\nx = 10\nx"),
            EvalResult::Val(Value::Int(10))
        );
    }

    #[test]
    fn test_lit_cannot_reassign() {
        let result = eval("lit x = 1\nx = 2");
        match result {
            EvalResult::Err(msg) => assert!(msg.contains("immutable")),
            _ => panic!("expected error"),
        }
    }

    // ── print ──

    #[test]
    fn test_print() {
        let output = eval_prints(r#"print("hello")"#);
        assert_eq!(output, vec!["hello"]);
    }

    #[test]
    fn test_print_multiple_args() {
        let output = eval_prints(r#"print("sum:", 1 + 2)"#);
        assert_eq!(output, vec!["sum: 3"]);
    }

    // ── 函數 ──

    #[test]
    fn test_function_call() {
        let input = r#"
fn addStore(a: Int, b: Int) -> Int {
    out a + b
}
addStore(3, 4)
"#;
        assert_eq!(eval(input), EvalResult::Val(Value::Int(7)));
    }

    #[test]
    fn test_function_wrong_arity() {
        let input = r#"
fn greetShow() -> String {
    out "hi"
}
greetShow(1)
"#;
        match eval(input) {
            EvalResult::Err(msg) => assert!(msg.contains("expected 0 arguments")),
            _ => panic!("expected error"),
        }
    }

    // ── if / else ──

    #[test]
    fn test_if_true() {
        let output = eval_prints(r#"if 5 > 3 { print("yes") }"#);
        assert_eq!(output, vec!["yes"]);
    }

    #[test]
    fn test_if_else() {
        let output = eval_prints(r#"if 1 > 10 { print("a") } else { print("b") }"#);
        assert_eq!(output, vec!["b"]);
    }

    // ── match ──

    #[test]
    fn test_match_string() {
        let input = r#"
lit val = "hello"
match val {
    is String s -> print(s)
    is Shadow e -> print(e.message)
}
"#;
        let output = eval_prints(input);
        assert_eq!(output, vec!["hello"]);
    }

    #[test]
    fn test_match_shadow() {
        let input = r#"
lit val = Shadow("oops")
match val {
    is String s -> print(s)
    is Shadow e -> print(e.message)
}
"#;
        let output = eval_prints(input);
        assert_eq!(output, vec!["oops"]);
    }

    // ── for ──

    #[test]
    fn test_for() {
        let input = r#"
lit items = [1, 2, 3]
for item in items {
    print(item)
}
"#;
        let output = eval_prints(input);
        assert_eq!(output, vec!["1", "2", "3"]);
    }

    // ── List / Map ──

    #[test]
    fn test_list_index() {
        assert_eq!(
            eval("lit arr = [10, 20, 30]\narr[1]"),
            EvalResult::Val(Value::Int(20))
        );
    }

    #[test]
    fn test_list_out_of_bounds_shadow() {
        let result = eval("lit arr = [1]\narr[5]");
        match result {
            EvalResult::Val(Value::Shadow(msg)) => assert!(msg.contains("out of bounds")),
            _ => panic!("expected Shadow for out of bounds"),
        }
    }

    #[test]
    fn test_map_access() {
        let input = r#"lit m = {"name": "Sunny"}
m["name"]"#;
        assert_eq!(
            eval(input),
            EvalResult::Val(Value::Str("Sunny".to_string()))
        );
    }

    // ── dot 屬性 ──

    #[test]
    fn test_shadow_message() {
        let input = r#"lit s = Shadow("err")
s.message"#;
        assert_eq!(
            eval(input),
            EvalResult::Val(Value::Str("err".to_string()))
        );
    }

    #[test]
    fn test_list_length() {
        assert_eq!(
            eval("lit arr = [1, 2, 3]\narr.length"),
            EvalResult::Val(Value::Int(3))
        );
    }

    // ── 整合測試：SPEC.md 範例 ──

    #[test]
    fn test_spec_example_data_show() {
        let input = r#"
fn dataShow(id: Int) -> String | Shadow {
    if id < 0 {
        out Shadow("ID must be positive")
    }
    out "Valid Data"
}

lit result = dataShow(-1)
match result {
    is String val -> print(val)
    is Shadow s -> print(s.message)
}
"#;
        let output = eval_prints(input);
        assert_eq!(output, vec!["ID must be positive"]);
    }

    #[test]
    fn test_spec_example_data_show_valid() {
        let input = r#"
fn dataShow(id: Int) -> String | Shadow {
    if id < 0 {
        out Shadow("ID must be positive")
    }
    out "Valid Data"
}

lit result = dataShow(5)
match result {
    is String val -> print(val)
    is Shadow s -> print(s.message)
}
"#;
        let output = eval_prints(input);
        assert_eq!(output, vec!["Valid Data"]);
    }

    // ══════════════════════════════════
    //  M2: List 方法
    // ══════════════════════════════════

    #[test]
    fn test_list_push() {
        assert_eq!(
            eval("lit a = [1, 2]\na.push(3)"),
            EvalResult::Val(Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]))
        );
    }

    #[test]
    fn test_list_pop() {
        assert_eq!(
            eval("lit a = [1, 2, 3]\na.pop()"),
            EvalResult::Val(Value::List(vec![Value::Int(1), Value::Int(2)]))
        );
    }

    #[test]
    fn test_list_pop_empty() {
        match eval("[].pop()") {
            EvalResult::Val(Value::Shadow(msg)) => assert!(msg.contains("empty")),
            other => panic!("expected Shadow, got {:?}", other),
        }
    }

    #[test]
    fn test_list_first_last() {
        assert_eq!(
            eval("[10, 20, 30].first()"),
            EvalResult::Val(Value::Int(10))
        );
        assert_eq!(
            eval("[10, 20, 30].last()"),
            EvalResult::Val(Value::Int(30))
        );
    }

    #[test]
    fn test_list_first_empty() {
        match eval("[].first()") {
            EvalResult::Val(Value::Shadow(_)) => {}
            other => panic!("expected Shadow, got {:?}", other),
        }
    }

    #[test]
    fn test_list_contains() {
        assert_eq!(
            eval("[1, 2, 3].contains(2)"),
            EvalResult::Val(Value::Bool(true))
        );
        assert_eq!(
            eval("[1, 2, 3].contains(9)"),
            EvalResult::Val(Value::Bool(false))
        );
    }

    #[test]
    fn test_list_reverse() {
        assert_eq!(
            eval("[1, 2, 3].reverse()"),
            EvalResult::Val(Value::List(vec![Value::Int(3), Value::Int(2), Value::Int(1)]))
        );
    }

    #[test]
    fn test_list_join() {
        assert_eq!(
            eval(r#"[1, 2, 3].join(", ")"#),
            EvalResult::Val(Value::Str("1, 2, 3".to_string()))
        );
    }

    // ══════════════════════════════════
    //  M2: Map 方法
    // ══════════════════════════════════

    #[test]
    fn test_map_keys() {
        let input = r#"lit m = {"a": 1, "b": 2}
m.keys()"#;
        assert_eq!(
            eval(input),
            EvalResult::Val(Value::List(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string()),
            ]))
        );
    }

    #[test]
    fn test_map_values() {
        let input = r#"lit m = {"x": 10, "y": 20}
m.values()"#;
        assert_eq!(
            eval(input),
            EvalResult::Val(Value::List(vec![Value::Int(10), Value::Int(20)]))
        );
    }

    #[test]
    fn test_map_has() {
        let input = r#"lit m = {"name": "Sunny"}
m.has("name")"#;
        assert_eq!(eval(input), EvalResult::Val(Value::Bool(true)));

        let input2 = r#"lit m = {"name": "Sunny"}
m.has("age")"#;
        assert_eq!(eval(input2), EvalResult::Val(Value::Bool(false)));
    }

    #[test]
    fn test_map_length_method() {
        let input = r#"lit m = {"a": 1, "b": 2}
m.length()"#;
        assert_eq!(eval(input), EvalResult::Val(Value::Int(2)));
    }

    #[test]
    fn test_map_length_property() {
        let input = r#"lit m = {"a": 1, "b": 2, "c": 3}
m.length"#;
        assert_eq!(eval(input), EvalResult::Val(Value::Int(3)));
    }

    #[test]
    fn test_map_remove() {
        let input = r#"lit m = {"a": 1, "b": 2}
m.remove("a")"#;
        assert_eq!(
            eval(input),
            EvalResult::Val(Value::Map(vec![("b".to_string(), Value::Int(2))]))
        );
    }

    // ══════════════════════════════════
    //  M2: String 方法
    // ══════════════════════════════════

    #[test]
    fn test_string_length_property() {
        assert_eq!(
            eval(r#""hello".length"#),
            EvalResult::Val(Value::Int(5))
        );
    }

    #[test]
    fn test_string_length_method() {
        assert_eq!(
            eval(r#""hello".length()"#),
            EvalResult::Val(Value::Int(5))
        );
    }

    #[test]
    fn test_string_trim() {
        assert_eq!(
            eval(r#""  hello  ".trim()"#),
            EvalResult::Val(Value::Str("hello".to_string()))
        );
    }

    #[test]
    fn test_string_upper_lower() {
        assert_eq!(
            eval(r#""hello".upper()"#),
            EvalResult::Val(Value::Str("HELLO".to_string()))
        );
        assert_eq!(
            eval(r#""HELLO".lower()"#),
            EvalResult::Val(Value::Str("hello".to_string()))
        );
    }

    #[test]
    fn test_string_contains() {
        assert_eq!(
            eval(r#""hello world".contains("world")"#),
            EvalResult::Val(Value::Bool(true))
        );
        assert_eq!(
            eval(r#""hello".contains("xyz")"#),
            EvalResult::Val(Value::Bool(false))
        );
    }

    #[test]
    fn test_string_starts_ends_with() {
        assert_eq!(
            eval(r#""hello".starts_with("he")"#),
            EvalResult::Val(Value::Bool(true))
        );
        assert_eq!(
            eval(r#""hello".ends_with("lo")"#),
            EvalResult::Val(Value::Bool(true))
        );
    }

    #[test]
    fn test_string_split() {
        assert_eq!(
            eval(r#""a,b,c".split(",")"#),
            EvalResult::Val(Value::List(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string()),
                Value::Str("c".to_string()),
            ]))
        );
    }

    // ══════════════════════════════════
    //  M2: Shadow 方法
    // ══════════════════════════════════

    #[test]
    fn test_shadow_wrap() {
        let input = r#"lit s = Shadow("not found")
s.wrap("DB")"#;
        assert_eq!(
            eval(input),
            EvalResult::Val(Value::Shadow("DB: not found".to_string()))
        );
    }

    #[test]
    fn test_shadow_unwrap_or() {
        let input = r#"lit s = Shadow("err")
s.unwrap_or("default")"#;
        assert_eq!(
            eval(input),
            EvalResult::Val(Value::Str("default".to_string()))
        );
    }

    // ══════════════════════════════════
    //  M2: 整合測試
    // ══════════════════════════════════

    #[test]
    fn test_m2_chain_list_methods() {
        // push then get length
        let input = r#"
lit items = [1, 2]
lit bigger = items.push(3)
bigger.length
"#;
        assert_eq!(eval(input), EvalResult::Val(Value::Int(3)));
    }

    #[test]
    fn test_m2_map_workflow() {
        let input = r#"
lit user = {"name": "Sunny", "role": "admin"}
print(user.has("name"))
print(user.keys().join(", "))
"#;
        let output = eval_prints(input);
        assert_eq!(output, vec!["true", "name, role"]);
    }

    #[test]
    fn test_m2_shadow_error_chain() {
        let input = r#"
fn findShow(id: Int) -> String | Shadow {
    if id < 0 {
        out Shadow("invalid id")
    }
    out "found"
}
lit result = findShow(-1)
match result {
    is String s -> print(s)
    is Shadow e -> print(e.wrap("findShow").message)
}
"#;
        let output = eval_prints(input);
        assert_eq!(output, vec!["findShow: invalid id"]);
    }

    // ══════════════════════════════════
    //  Range 語法
    // ══════════════════════════════════

    #[test]
    fn test_range_basic() {
        assert_eq!(
            eval("0..5"),
            EvalResult::Val(Value::List(vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
            ]))
        );
    }

    #[test]
    fn test_range_empty() {
        assert_eq!(eval("5..5"), EvalResult::Val(Value::List(vec![])));
    }

    #[test]
    fn test_for_range() {
        let input = r#"
glow sum = 0
for i in 0..5 {
    sum = sum + i
}
sum
"#;
        assert_eq!(eval(input), EvalResult::Val(Value::Int(10)));
    }

    #[test]
    fn test_for_range_with_variables() {
        let input = r#"
lit start = 1
lit end = 4
glow total = 0
for i in start..end {
    total = total + i
}
total
"#;
        assert_eq!(eval(input), EvalResult::Val(Value::Int(6)));
    }

    // ══════════════════════════════════
    //  閉包 / 匿名函數
    // ══════════════════════════════════

    #[test]
    fn test_closure_basic() {
        let input = r#"
lit add = fn(x, y) { out x + y }
add(3, 4)
"#;
        assert_eq!(eval(input), EvalResult::Val(Value::Int(7)));
    }

    #[test]
    fn test_closure_no_params() {
        let input = r#"
lit greetShow = fn() { out "hello" }
greetShow()
"#;
        assert_eq!(
            eval(input),
            EvalResult::Val(Value::Str("hello".to_string()))
        );
    }

    #[test]
    fn test_closure_as_argument() {
        let input = r#"
fn applyShow(f: Int, x: Int) -> Int {
    out f(x)
}
lit double = fn(n) { out n * 2 }
applyShow(double, 5)
"#;
        assert_eq!(eval(input), EvalResult::Val(Value::Int(10)));
    }
}

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// ray 執行緒結果：完成後存放 Value 或錯誤訊息
#[derive(Debug, Clone)]
pub struct RayResult {
    pub value: Arc<Mutex<Option<Result<Value, String>>>>,
    pub handle: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
}

/// Sunny 運行時的值類型
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    Map(Vec<(String, Value)>),
    Shadow(String),
    /// 函數值：參數名列表 + 函數體（以 AST 語句儲存）
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<crate::ast::Statement>,
    },
    /// ray 併發句柄
    RayHandle(RayResult),
    /// 空值（無回傳的表達式結果）
    Void,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Shadow(a), Value::Shadow(b)) => a == b,
            (Value::Void, Value::Void) => true,
            (Value::Function { name: a, .. }, Value::Function { name: b, .. }) => a == b,
            (Value::RayHandle(_), Value::RayHandle(_)) => false, // 句柄不可比較
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Str(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::List(items) => {
                let parts: Vec<String> = items.iter().map(|i| format!("{}", i)).collect();
                write!(f, "[{}]", parts.join(", "))
            }
            Value::Map(pairs) => {
                let parts: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v))
                    .collect();
                write!(f, "{{{}}}", parts.join(", "))
            }
            Value::Shadow(msg) => write!(f, "Shadow({})", msg),
            Value::Function { name, params, .. } => {
                write!(f, "fn {}({})", name, params.join(", "))
            }
            Value::RayHandle(_) => write!(f, "<ray>"),
            Value::Void => write!(f, "void"),
        }
    }
}

/// 變數綁定的元資料
#[derive(Debug, Clone)]
struct Binding {
    value: Value,
    mutable: bool, // true = glow, false = lit
}

/// 作用域環境：帶有外層指針的變數映射表
#[derive(Debug, Clone)]
pub struct Environment {
    store: HashMap<String, Binding>,
    outer: Option<Box<Environment>>,
}

impl Environment {
    /// 建立全域環境
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    /// 建立子作用域（進入函數/區塊時使用）
    pub fn enclosed(outer: Environment) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(Box::new(outer)),
        }
    }

    /// `lit` 綁定 — 不可變
    pub fn define_lit(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.store.contains_key(name) {
            return Err(format!("'{}' is already defined in this scope", name));
        }
        self.store.insert(
            name.to_string(),
            Binding {
                value,
                mutable: false,
            },
        );
        Ok(())
    }

    /// `glow` 綁定 — 可變
    pub fn define_glow(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.store.contains_key(name) {
            return Err(format!("'{}' is already defined in this scope", name));
        }
        self.store.insert(
            name.to_string(),
            Binding {
                value,
                mutable: true,
            },
        );
        Ok(())
    }

    /// 重新賦值 — 僅限 glow 變數
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if let Some(binding) = self.store.get_mut(name) {
            if !binding.mutable {
                return Err(format!(
                    "cannot reassign '{}': lit bindings are immutable (the sun never changes)",
                    name
                ));
            }
            binding.value = value;
            return Ok(());
        }
        // 往外層作用域查找
        if let Some(outer) = &mut self.outer {
            return outer.assign(name, value);
        }
        Err(format!("'{}' is not defined", name))
    }

    /// 取出外層環境（用於 for 迴圈回寫修改）
    pub fn take_outer(&mut self) -> Option<Environment> {
        self.outer.take().map(|b| *b)
    }

    /// 取得變數值
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(binding) = self.store.get(name) {
            return Some(binding.value.clone());
        }
        if let Some(outer) = &self.outer {
            return outer.get(name);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_lit_and_get() {
        let mut env = Environment::new();
        env.define_lit("x", Value::Int(42)).unwrap();
        assert_eq!(env.get("x"), Some(Value::Int(42)));
    }

    #[test]
    fn test_define_glow_and_get() {
        let mut env = Environment::new();
        env.define_glow("count", Value::Int(0)).unwrap();
        assert_eq!(env.get("count"), Some(Value::Int(0)));
    }

    #[test]
    fn test_lit_cannot_reassign() {
        let mut env = Environment::new();
        env.define_lit("x", Value::Int(42)).unwrap();
        let result = env.assign("x", Value::Int(99));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("immutable"));
        // 值不變
        assert_eq!(env.get("x"), Some(Value::Int(42)));
    }

    #[test]
    fn test_glow_can_reassign() {
        let mut env = Environment::new();
        env.define_glow("count", Value::Int(0)).unwrap();
        env.assign("count", Value::Int(10)).unwrap();
        assert_eq!(env.get("count"), Some(Value::Int(10)));
    }

    #[test]
    fn test_duplicate_define_rejected() {
        let mut env = Environment::new();
        env.define_lit("x", Value::Int(1)).unwrap();
        let result = env.define_lit("x", Value::Int(2));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already defined"));
    }

    #[test]
    fn test_enclosed_scope_inherits() {
        let mut outer = Environment::new();
        outer.define_lit("x", Value::Int(42)).unwrap();

        let inner = Environment::enclosed(outer);
        // 子作用域可以讀到外層變數
        assert_eq!(inner.get("x"), Some(Value::Int(42)));
    }

    #[test]
    fn test_enclosed_scope_shadows() {
        let mut outer = Environment::new();
        outer.define_lit("x", Value::Int(1)).unwrap();

        let mut inner = Environment::enclosed(outer);
        inner.define_lit("x", Value::Int(99)).unwrap();
        // 子作用域的值優先
        assert_eq!(inner.get("x"), Some(Value::Int(99)));
    }

    #[test]
    fn test_assign_reaches_outer_scope() {
        let mut outer = Environment::new();
        outer.define_glow("count", Value::Int(0)).unwrap();

        let mut inner = Environment::enclosed(outer);
        inner.assign("count", Value::Int(5)).unwrap();
        // 透過 inner 修改了 outer 的 glow 變數
        assert_eq!(inner.get("count"), Some(Value::Int(5)));
    }

    #[test]
    fn test_assign_undefined_error() {
        let mut env = Environment::new();
        let result = env.assign("ghost", Value::Int(1));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not defined"));
    }

    #[test]
    fn test_get_undefined_returns_none() {
        let env = Environment::new();
        assert_eq!(env.get("nothing"), None);
    }

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::Int(42)), "42");
        assert_eq!(format!("{}", Value::Float(3.14)), "3.14");
        assert_eq!(format!("{}", Value::Str("hello".into())), "hello");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Shadow("err".into())), "Shadow(err)");
        assert_eq!(format!("{}", Value::Void), "void");
        assert_eq!(
            format!("{}", Value::List(vec![Value::Int(1), Value::Int(2)])),
            "[1, 2]"
        );
    }
}

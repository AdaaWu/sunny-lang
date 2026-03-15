# 語法參考

## 關鍵字

| 關鍵字 | 用途 |
|--------|------|
| `lit` | 不可變綁定 |
| `glow` | 可變綁定 |
| `fn` | 函數定義 |
| `out` | 回傳值 |
| `if` / `else` | 條件判斷 |
| `for` / `in` | 迴圈迭代 |
| `while` | 條件迴圈 |
| `match` / `is` | 模式匹配 |
| `ray` | 併發區塊 |
| `import` | 匯入模組 |
| `and` / `or` / `not` | 邏輯運算子 |
| `true` / `false` | 布林字面量 |

## 運算子

| 運算子 | 說明 |
|--------|------|
| `+` `-` `*` `/` `%` | 算術 |
| `==` `!=` `<` `>` `<=` `>=` | 比較 |
| `and` / `&&` | 邏輯與 |
| `or` / `||` | 邏輯或 |
| `not` / `!` | 邏輯非 |
| `..` | Range（`0..10`） |
| `.` | 屬性/方法存取 |
| `->` | 函數回傳型別標註 |

## 變數綁定

```sunny
lit x = 42        // 不可變
glow y = 0        // 可變
y = y + 1         // OK
```

## 函數定義

```sunny
// 具名函數（需要 Resource Action 後綴）
fn userShow(id: Int) -> String | Shadow {
    out "user-" + to_string(id)
}

// 匿名函數（閉包）
lit add = fn(a, b) { out a + b }
```

## 控制流程

```sunny
// if / else
if condition {
    // ...
} else {
    // ...
}

// for..in
for item in list { }
for i in 0..10 { }

// while
while condition { }

// match
match value {
    is String s -> print(s)
    is Shadow e -> print(e.message)
}
```

## 字串

```sunny
lit s1 = "double quotes"
lit s2 = 'single quotes'
lit name = "Sunny"
lit msg = "Hello, {name}!"   // 字串內插
```

## 註解

```sunny
// 單行註解

/* 多行
   註解 */
```

## 模組

```sunny
import "utils.sunny"
```

## 命名規範

| 對象 | 規範 | 範例 |
|------|------|------|
| 變數/函數 | camelCase | `userName`, `orderStore()` |
| 常數 | SCREAMING_SNAKE_CASE | `MAX_RETRY` |
| 型別 | PascalCase | `Int`, `String` |

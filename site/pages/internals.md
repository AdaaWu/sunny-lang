# 實作原理

Sunny 的整條工具鏈（約 6,300 行 Rust，180 個測試）都是手寫的，沒有 parser generator，也沒有第三方 runtime。這一頁說明它怎麼運作。

## 管線總覽

```
原始碼 ──▶ Lexer ──▶ Token 串流 ──▶ Parser ──▶ AST ──▶ Linter ──▶ Evaluator ──▶ 結果
                                                                      │
                                                  Markdown / Template / HTTP Server
```

| 檔案 | 職責 |
|------|------|
| `src/token.rs`、`src/lexer.rs` | Token 定義與逐字元掃描，支援 `//`、`/* */` 註解與單雙引號字串 |
| `src/parser.rs`、`src/ast.rs` | Pratt Parser，產生 Statement / Expression 組成的 AST |
| `src/linter.rs` | 檢查具名函數是否帶有 Resource Action 後綴 |
| `src/evaluator.rs`、`src/environment.rs` | 樹走訪求值器與作用域鏈 |
| `src/markdown.rs`、`src/template.rs` | 內建 Markdown 與模板引擎 |
| `src/router.rs`、`src/server.rs` | 函數名稱到 HTTP 路由的映射與伺服器 |
| `src/wasm.rs` | 瀏覽器入口：`eval_sunny`、`tokenize_sunny` |

## Pratt Parsing

運算子優先級用一個 enum 表達，共 9 個層級，數字越大綁得越緊：

```
Or < And < Equals < LessGt < Sum < Product < Prefix < Call < Index
```

核心只有一個迴圈：先解析前綴（數字、括號、`-x` 等），然後只要「下一個運算子的優先級比目前高」，就把左邊吞進去組成中綴節點。

```rust
fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
    let mut left = self.parse_prefix()?;
    while precedence < self.cur_precedence() {
        left = /* 依 Token 選擇 infix / call / index / range 解析 */;
    }
    Some(left)
}
```

要新增運算子，只需要在 `token_precedence` 加一行並補上對應的解析函數。

## 作用域鏈

`Environment` 是一個 HashMap 加上指向外層的指標。查找變數時從目前作用域往外走；每個綁定記錄自己是 `lit`（不可變）還是 `glow`（可變）。閉包捕捉的就是建立當下的環境，所以 `fn(x) { out x * factor }` 能記住 `factor`。

## Shadow：沒有 exception 的錯誤處理

求值器的回傳型別是：

```
EvalResult = Val(值) | Output(out 的回傳值，向上冒泡) | Err(內部錯誤)
```

`Err` 只代表語言本身的問題（例如參數個數錯誤）。使用者程式會遇到的失敗，像除以零、陣列越界、找不到 key、讀不到檔案，都是一般的 `Shadow` 值，例如 `Shadow(index 99 out of bounds (length 2))`。因為它是值，所以沒有隱形的控制流程：呼叫者必須用 `match`、`.wrap` 或 `.unwrap_or` 明確處理。詳見 [Shadow 錯誤處理](shadow.html)。

## 自舉：用 Sunny 產生這個網站

文檔站不是靠 Node 或 Hugo，而是一支 Sunny 程式。Markdown 轉換和模板替換是求值器的內建函數 `render_md` 與 `render_template`，`site/site.sunny` 負責串起來：

```sunny
lit layout = read_file("site/layouts/base.html")
lit pages = ["index", "tutorial", "syntax", "types", "builtins", "shadow", "server", "internals", "playground"]

for name in pages {
    lit md = read_file("site/pages/" + name + ".md")
    lit html = render_md(md)
    lit page = render_template(layout, {
        "title": name,
        "content": html
    })
    write_file("dist/" + name + ".html", page)
}
```

（上面是精簡版；實際檔案還會為每頁產生導覽列、上下頁連結與 meta 資訊，完整內容見 [GitHub 上的 site.sunny](https://github.com/AdaaWu/sunny-lang/blob/main/site/site.sunny)。）

## WebAssembly

Cargo 以 feature 切開原生與瀏覽器：`native` 包含 HTTP Server，`wasm` 只帶 `wasm-bindgen`。CI 用 `wasm-pack build --target web -- --no-default-features --features wasm` 編出 `sunny_lang_bg.wasm`，Playground 的 `runner.js` 載入後呼叫 `eval_sunny` 執行程式、`tokenize_sunny` 取得 Token。

瀏覽器沒有檔案系統、執行緒與系統時鐘，所以求值器在 `wasm32` 下做了三個調整：`read_file` / `write_file` 回傳 Shadow、`ray { }` 同步執行、`time_now()` 回傳 0。詳見 [Playground 說明](playground.html)。

[在 Playground 試試看](playground/index.html)

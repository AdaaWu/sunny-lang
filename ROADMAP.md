# ☀️ Sunny-lang → 自舉 Playground 網站 全攻略

> 最終目標：用 Sunny 語言自己寫出一個可互動的官方文檔 + 線上 Playground 網站。

---

## 現狀盤點 (已完成)

| 里程碑 | 狀態 | 內容 |
|--------|------|------|
| M1 核心引擎 | ✅ | Token → Lexer → Parser → AST → Evaluator → Linter → REPL |
| M2 資料與安全 | ✅ | List/Map/String 方法、Shadow 錯誤鏈 |
| M3 後端動力 | ✅ | ray 併發、自動路由映射、HTTP Server、CLI serve/restart/stop |
| M4 自給自足 | ✅ | Markdown 渲染、Template 引擎、docs.sunny 文檔系統 |
| 語言增強 | ✅ | for..in range(`0..10`)、閉包(`fn(x) { ... }`)、import 模組系統、`..` 語法 |

**測試數量：157 tests，全部通過**

---

## Phase 5：語言補強 — 自舉前必備能力

> 目前 Sunny 跑得動 demo，但要撐起真正的 Playground，還缺以下能力。

### 5.1 字串內插 (String Interpolation)

```sunny
lit name = "Sunny"
lit msg = "Hello, {name}!"    // → "Hello, Sunny!"
```

- Token: 在 lexer 解析 `"..."` 時偵測 `{expr}` 區段
- AST: 新增 `Expression::StringInterpolation(Vec<Expression>)`
- Evaluator: 逐段求值後拼接

### 5.2 while 迴圈

```sunny
glow i = 0
while i < 10 {
    print(i)
    i = i + 1
}
```

- 新增 `while` keyword → Token/AST/Parser/Evaluator

### 5.3 多行函數體回傳 (隱式 out)

```sunny
fn addShow(a: Int, b: Int) -> Int {
    a + b   // 最後一個表達式即回傳值
}
```

- Evaluator: 若函數體最後一條是 Expression statement，自動視為 out

### 5.4 型別轉換內建函數

```sunny
lit n = to_int("42")       // String → Int
lit s = to_string(42)      // Int → String
lit f = to_float(42)       // Int → Float
```

### 5.5 更多內建函數

| 函數 | 用途 |
|------|------|
| `len(x)` | 統一取長度（List/String/Map） |
| `type_of(x)` | 回傳型別名稱字串 |
| `json_encode(map)` | Map → JSON String |
| `json_decode(str)` | JSON String → Map |
| `time_now()` | 當前 Unix timestamp (Int) |
| `env(key)` | 讀取環境變數 |

---

## Phase 6：WebAssembly 編譯 — Playground 核心引擎

> 讓 Sunny 的 Evaluator 能在瀏覽器跑，不依賴後端。

### 6.1 WASM Target

```
# 安裝 wasm-pack
cargo install wasm-pack

# 新增 crate-type
[lib]
crate-type = ["cdylib", "rlib"]
```

- 抽出 `lib.rs`：把 token/lexer/parser/ast/evaluator/environment 打包成 library crate
- `main.rs` 只保留 CLI 入口
- 排除 tiny_http、std::fs、std::thread 等非 WASM 相容功能（用 `#[cfg(not(target_arch = "wasm32"))]`）

### 6.2 WASM 公開介面

```rust
// src/wasm.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn eval_sunny(source: &str) -> String {
    // parse → eval → 收集 output_buffer → 回傳 JSON
}

#[wasm_bindgen]
pub fn tokenize_sunny(source: &str) -> String {
    // lex → 回傳 token 列表 JSON（供語法高亮用）
}
```

### 6.3 產出

```
wasm-pack build --target web --out-dir playground/pkg
```

產出 `playground/pkg/sunny_lang.js` + `.wasm`，可直接被前端 import。

---

## Phase 7：Playground 前端

> 純靜態 HTML/CSS/JS，用 Sunny WASM 在瀏覽器端即時執行。

### 7.1 專案結構

```
playground/
├── index.html          # 主頁面
├── style.css           # 樣式（深色主題）
├── editor.js           # 程式碼編輯器邏輯
├── runner.js           # 呼叫 WASM eval_sunny()
├── examples.js         # 預設範例程式碼
├── syntax.js           # 語法高亮（呼叫 tokenize_sunny()）
└── pkg/                # wasm-pack 產出
    ├── sunny_lang.js
    └── sunny_lang_bg.wasm
```

### 7.2 核心功能

| 功能 | 說明 |
|------|------|
| 程式碼編輯區 | Monospace textarea + 行號 + Tab 支援 |
| 即時執行 | 按 ▶ Run 或 Ctrl+Enter，呼叫 WASM eval |
| 輸出面板 | 顯示 print 輸出、return 值、錯誤訊息 |
| 語法高亮 | 即時 tokenize → 替 keyword/string/number 上色 |
| 範例選單 | 下拉選擇：Hello World / List 操作 / Shadow 處理 / For Range / Closure |
| 分享連結 | 程式碼 base64 encode → URL hash → 可分享 |
| 響應式 | 桌面左右分欄、手機上下分欄 |

### 7.3 語法高亮配色

```
keyword  (lit/glow/fn/out/if/else/for/in/match/is/ray/import) → #ff79c6 粉紅
string   ("..." / '...')                                       → #f1fa8c 黃色
number   (42 / 3.14)                                           → #bd93f9 紫色
comment  (// ... / /* ... */)                                  → #6272a4 灰藍
builtin  (print/await/Shadow/read_file)                        → #50fa7b 綠色
type     (Int/Float/String/Bool/List/Map/Shadow)               → #8be9fd 青色
ident    (camelCase)                                            → #f8f8f2 白色
```

---

## Phase 8：官方文檔網站 — 用 Sunny 自舉

> M4 已有 Markdown 渲染和 Template 引擎，這裡把它做成完整的文檔系統。

### 8.1 網站結構

```
site/
├── pages/
│   ├── index.md          # 首頁（語言介紹、快速開始）
│   ├── tutorial.md       # 教學（逐步引導）
│   ├── syntax.md         # 語法參考
│   ├── types.md          # 型別系統
│   ├── builtins.md       # 內建函數列表
│   ├── shadow.md         # Shadow 錯誤處理哲學
│   ├── server.md         # HTTP Server 指南
│   └── playground.md     # Playground 使用說明
├── layouts/
│   └── base.html         # HTML 模板（含 nav/sidebar/footer）
├── static/
│   ├── style.css
│   └── logo.svg
└── site.sunny            # Sunny 寫的靜態站產生器
```

### 8.2 site.sunny — 核心自舉邏輯

```sunny
// 讀取所有 .md 頁面，渲染成 HTML，嵌入 layout

fn pagesIndex() -> String {
    lit layout = read_file("layouts/base.html")
    lit files = ["index", "tutorial", "syntax", "types", "builtins", "shadow", "server", "playground"]

    for name in files {
        lit md = read_file("pages/" + name + ".md")
        lit html = render_md(md)
        lit page = render_template(layout, {
            "title": name,
            "content": html,
            "nav_active": name
        })
        write_file("dist/" + name + ".html", page)
    }
    out "Built all pages"
}
```

需新增的內建函數：
- `write_file(path, content)` — 寫入檔案

### 8.3 Playground 嵌入

文檔頁面的程式碼範例旁邊加上「▶ Try it」按鈕：

```html
<pre class="sunny-code" data-runnable="true">
lit items = [1, 2, 3]
for item in items {
    print(item * 2)
}
</pre>
```

JavaScript 偵測 `data-runnable`，點擊後開啟 inline Playground 面板，用 WASM 即時執行。

---

## Phase 9：部署與發佈

### 9.1 建置流程

```bash
# 1. 編譯 WASM
wasm-pack build --target web --out-dir playground/pkg

# 2. 用 Sunny 自己產生文檔頁面
sun site/site.sunny

# 3. 複製 Playground 靜態檔到 dist
cp -r playground/ dist/playground/

# 4. 部署到 GitHub Pages / Cloudflare Pages / Vercel
```

### 9.2 CI/CD (GitHub Actions)

```yaml
name: Build & Deploy
on:
  push:
    branches: [main]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install wasm-pack
      - run: cargo test
      - run: wasm-pack build --target web --out-dir playground/pkg
      - run: cargo run -- site/site.sunny
      - uses: peaceiris/actions-gh-pages@v3
        with:
          publish_dir: ./dist
```

---

## 執行順序 & 優先級

```
Phase 5 語言補強          ← 現在開始
  ↓
Phase 6 WASM 編譯         ← 讓 Sunny 跑在瀏覽器
  ↓
Phase 7 Playground 前端    ← 互動式程式碼執行
  ↓
Phase 8 文檔網站自舉       ← 用 Sunny 寫 Sunny 的官方網站
  ↓
Phase 9 部署上線           ← GitHub Pages / Cloudflare
```

### 各 Phase 預估工作量

| Phase | 核心改動 | 新增檔案 |
|-------|---------|---------|
| 5 語言補強 | token/lexer/parser/ast/evaluator | — |
| 6 WASM | lib.rs, wasm.rs, Cargo.toml | 2 |
| 7 Playground | index.html, editor.js, runner.js, syntax.js, style.css, examples.js | 6 |
| 8 文檔自舉 | site.sunny, pages/*.md, layouts/base.html | ~12 |
| 9 部署 | .github/workflows/deploy.yml | 1 |

---

## 最終願景

```
                    ┌──────────────────────────┐
                    │   sunny-lang.dev          │
                    │                           │
  ┌─────────┐      │  ┌───────┐  ┌──────────┐ │
  │ .sunny  │──▶   │  │ Docs  │  │Playground│ │
  │ source  │  sun  │  │(自舉) │  │ (WASM)   │ │
  └─────────┘      │  └───────┘  └──────────┘ │
                    │                           │
                    │  Powered by Sunny ☀️      │
                    └──────────────────────────┘
```

Sunny 語言寫出自己的文檔、Sunny 語言的 Evaluator 跑在瀏覽器裡讓人即時試玩 — 這就是自舉。

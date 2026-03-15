# 🎯 Sunny Language 自給自足計畫

1. **語言核心**：具備強型別檢查、`fn` 資源命名規範、以及 `Shadow` 錯誤處理機制。
2. **Web 框架**：內建名為 `Solaris` 的 Web 引擎，能自動將 `fn userIndex()` 映射為 API 路由。
3. **開源文檔站**：使用 Sunny 撰寫文檔的後端邏輯與前端組件，實現「用 Sunny 解釋 Sunny」。

---

## 📅 開發時程表 (2026)

### 第一階段：地基工程 (Month 1 - 核心直譯器)

**目標：讓電腦聽懂 Sunny 的基本邏輯。**

* **第 1-2 週**：完成 `Lexer` 與 `Parser`。支援 `lit`、`glow`、`fn` 及基礎運算。
* **第 3 週**：實作 `Evaluator` (求值引擎) 與作用域管理。
* **第 4 週**：**規範檢查器 (The Linter)**。強制執行 `snakeCase` (如 `userName`) 與 Resource Action (如 `bookIndex`) 的命名檢查。如果不符合規範，編譯直接攔截。

### 第二階段：數據與結構 (Month 2 - Web 數據核心)

**目標：支援網頁開發必備的 JSON 與 複合型別。**

* **第 1-2 週**：實作 `List` 與 `Map`。這將是我們處理網頁數據的基石。
* **第 3 週**：完善 `Shadow` 類型傳遞邏輯。確保程式碼不會發生未預期的崩潰。
* **第 4 週**：實作 JSON 解析與生成模組。

### 第三階段：動力與網路 (Month 3 - 後端與併發)

**目標：讓 Sunny 具備伺服器能力。**

* **第 1-2 週**：封裝 Rust 的網路庫，提供 `fn netStore()` 類的標準通訊介面。
* **第 3 週**：實作 **`Ray`** 輕量級併發。讓你的文檔站能同時應付大量讀者。
* **第 4 週**：**自動路由映射系統**。這會是 Sunny 的殺手鐧：只要寫好 `fn docShow(id: Int)`，路由就會自動對應到 `/doc/{id}`。

### 第四階段：實作與開源 (Month 4 - 文檔系統建立)

**目標：用 Sunny 寫出自己的官方網站。**

* **第 1-2 週**：開發文檔站的後端邏輯與 Markdown 解析引擎（用 Sunny 寫）。
* **第 3 週**：前端整合（透過 Wasm 或轉譯為 JavaScript）。
* **第 4 週**：正式將原始碼與文檔發佈到 GitHub。

---

## 🏗️ 最終成品架構預覽

當這 4 個月結束時，你的 `docs.sunny-lang.org` 會是這樣運作的：

```sunny
// 官方文檔資料庫的操作
fn docIndex() -> List | Shadow {
    lit data = fileRead("docs/getting-started.md")
    // 邏輯處理...
    output data
}

// 當使用者訪問 /doc 時，自動觸發上面的函數

```

---

## 🚀 踏出第一步：定義 Token 類型

既然時程與目標都明確了，我們現在就開始執行 **第一階段的第一週**。我們需要在 Rust 中建立 `src/token.rs`。

這是目前我們確定的關鍵 Token：

* **關鍵字**：`fn`, `lit`, `glow`, `if`, `otherwise`, `cycle`, `in`, `match`, `is`, `output`
* **操作符**：`->`, `=`, `==`, `+`, `-`, `*`, `/`
* **定界符**：`{`, `}`, `(`, `)`, `:`, `,`


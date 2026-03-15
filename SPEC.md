這是一份為 **Sunny** 程式語言量身打造的精簡版規格文件（v1.3）。這份文件將作為我們未來 4 個月開發的最高準則。

---

# ☀️ Sunny 語言程式規格書 (Technical Specification)

## 1. 核心願景

* **目標**：成為一個具備高度工程規範的全端語言，能開發高性能後端 API 與 WebAssembly 前端應用。
* **最終指標**：使用 Sunny 語言開發其自身的官方開源文檔系統。

## 2. 命名規範 (Strict Naming)

Sunny 透過強制性的命名規則確保程式碼的易讀性與一致性：

* **變數與函數**：統一使用 `camelCase` (小駝峰)。
* **常數**：使用 `SCREAMING_SNAKE_CASE` (全大寫底線)。
* **類型名稱**：使用 `PascalCase` (大駝峰)。
* **Resource Action (核心規定)**：
全局或模組函數必須以資源行為作字尾，否則檢查器將拒絕執行：
* `index`: 取得列表（例如：`userIndex`）
* `show`: 取得單一資料（例如：`bookShow`）
* `store`: 儲存新資料（例如：`orderStore`）
* `update`: 更新現有資料（例如：`statusUpdate`）
* `remove`: 刪除資料（例如：`fileRemove`）



## 3. 語法結構 (Syntax)

### 3.1 變數定義

* **`lit`** (Literal/Constant): 定義後不可修改。
* **`glow`** (Variable): 允許重新賦值。

### 3.2 函數定義

使用 **`fn`** 關鍵字，必須標註回傳類型與使用 `output` 返回結果。

```sunny
fn bookShow(id: Int) -> String | Shadow {
    lit bookName = "Sunny Guide"
    output bookName
}

```

## 4. 數據類型 (Data Types)

| 類型 | 說明 | 範例 |
| --- | --- | --- |
| `Int` / `Float` | 64 位元數值 | `100`, `3.14` |
| `String` / `Bool` | 文字與布林 | `"Hello"`, `true` |
| `List` | 有序集合 (Array) | `[1, 2, 3]` |
| `Map` | 鍵值對 (Object) | `{"id": 1, "name": "Sunny"}` |
| `Shadow` | 錯誤或空值封裝 | `Shadow("Not Found")` |

## 5. 流程控制 (Logic Control)

* **條件**：使用 `if` 與 `otherwise`。
* **循環**：使用 `cycle .. in ..` 結構。
* **錯誤處理**：強制使用 `match` 處理 `Shadow` 類型，杜絕隱藏的崩潰風險。

## 6. 系統功能 (Advanced)

* **併發處理**：使用 `ray { ... }` 啟動輕量級非同步任務。
* **自動路由**：後端框架自動根據 `fn` 名稱映射 HTTP 路由（如 `userIndex` 映射至 `GET /user`）。

## 7. 開發時程 (Roadmap)

| 階段 | 週期 | 核心任務 |
| --- | --- | --- |
| **M1: 核心引擎** | 4 週 | 完成 Rust 版 Lexer/Parser/Evaluator，實作 Linter 命名檢查。 |
| **M2: Web 資料** | 4 週 | 支援 `List`, `Map` 與原生的 JSON 處理機制。 |
| **M3: 網路併發** | 4 週 | 實作 `Ray` 併發模型與自動路由映射伺服器。 |
| **M4: 自給自足** | 4 週 | 用 Sunny 撰寫並發佈官方開源文檔系統。 |

---

> **開發筆記**：Sunny 拒絕任何「黑箱」邏輯，所有行為必須顯式定義，確保開發過程如陽光般透明。

---

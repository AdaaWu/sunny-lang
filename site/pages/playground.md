# Playground

Sunny Playground 讓你直接在瀏覽器中編寫和執行 Sunny 程式碼。

## 使用方式

1. 在左側編輯器輸入 Sunny 程式碼
2. 按下 **Run** 按鈕或 `Ctrl+Enter` 執行
3. 右側面板顯示輸出結果

## 功能

- **即時執行** — 程式碼在瀏覽器中透過 WebAssembly 執行，不需要後端
- **語法高亮** — 即時標記關鍵字、字串、數字等
- **範例選單** — 下拉選擇預設範例快速體驗
- **分享連結** — 點擊 Share 按鈕複製帶有程式碼的 URL

## 快速鍵

| 快速鍵 | 動作 |
|--------|------|
| `Ctrl+Enter` / `Cmd+Enter` | 執行程式碼 |
| `Tab` | 插入 4 個空格 |

## 限制

Playground 在瀏覽器中執行，以下功能不可用：

- `read_file()` / `write_file()` — 檔案系統操作
- `ray { }` — 多執行緒併發
- HTTP Server 相關功能

所有純計算、字串處理、List/Map 操作、閉包等都可以正常使用。

## 開啟 Playground

[點此開啟 Playground](../playground/index.html)

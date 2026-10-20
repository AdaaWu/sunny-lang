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

Playground 由同一個求值器編譯成 WebAssembly，在瀏覽器沙盒中執行，所以有些行為和命令列的 `sun` 不同：

| 功能 | 在 Playground 的行為 |
|------|----------------------|
| `read_file()` / `write_file()` | 瀏覽器沒有檔案系統，一律回傳 Shadow（不會當掉） |
| `ray { }` | 同步執行（沒有執行緒），結果相同但不會平行 |
| `time_now()` | 沒有系統時鐘，固定回傳 `0` |
| HTTP Server（`sun serve`） | 不可用，那是命令列工具 |
| Resource Action Linter | 不會執行，所以函數命名不會被檢查；用 `sun` 執行時才會中止 |

所有純計算、字串處理、List/Map 操作、閉包、`match` 與 Shadow 都可以正常使用。

## 開啟 Playground

[點此開啟 Playground](playground/index.html)

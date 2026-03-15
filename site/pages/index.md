# Sunny Lang

> 像陽光一樣透明的程式語言。

Sunny 是一門用 Rust 打造的全端程式語言，強制工程規範、拒絕黑箱邏輯。

## 特色

- **不可變優先** — `lit` 定義常數，`glow` 才允許修改
- **Shadow 錯誤處理** — 用型別取代 null 和 exception，強制你面對問題
- **Resource Action** — 函數命名必須帶動詞後綴（`index`/`show`/`store`/`update`/`remove`）
- **自動路由** — `fn productIndex()` 自動映射為 `GET /product`
- **ray 併發** — 輕量級執行緒模型
- **自舉** — 這個文檔網站就是用 Sunny 產生的

## 快速開始

```sunny
lit name = "World"
print("Hello, {name}!")

lit items = [1, 2, 3, 4, 5]
for item in items {
    print(item * 2)
}
```

## 安裝

```bash
git clone https://github.com/user/sunny-lang.git
cd sunny-lang
cargo build --release
```

執行檔位於 `target/release/sun`，可以加入 PATH。

```bash
sun hello.sunny          # 執行檔案
sun                      # 進入 REPL
sun serve app.sunny      # 啟動 HTTP Server
```

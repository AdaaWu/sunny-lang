# HTTP Server

Sunny 內建 HTTP Server，函數名稱自動映射為 REST 路由。

## 啟動

```bash
sun serve app.sunny          # port 3000
sun serve app.sunny 8080     # 自訂 port
sun restart app.sunny 3000   # 重啟
sun stop 3000                # 停止
```

## 自動路由

函數名稱的 Resource Action 後綴決定 HTTP Method 和路徑：

| 後綴 | HTTP Method | 路徑範例 |
|------|-------------|----------|
| `index` | GET | `/resource` |
| `show` | GET | `/resource/:id` |
| `store` | POST | `/resource` |
| `update` | PUT | `/resource/:id` |
| `remove` | DELETE | `/resource/:id` |

## 範例 API

```sunny
fn productIndex() -> List {
    out [
        {"id": 1, "name": "T-Shirt", "price": 29},
        {"id": 2, "name": "Mug", "price": 15}
    ]
}

fn productShow(id: Int) -> Map | Shadow {
    if id == 1 {
        out {"id": 1, "name": "T-Shirt", "price": 29}
    }
    out Shadow("product not found")
}

fn healthIndex() -> Map {
    out {"status": "ok", "lang": "Sunny"}
}
```

啟動後自動產生路由表：

```
Routes:
  GET /product     -> productIndex()
  GET /product/:id -> productShow()
  GET /health      -> healthIndex()
```

## 回應格式

- 回傳 Map/List → 自動轉 JSON（`Content-Type: application/json`）
- 回傳包含 `<` 的 String → 視為 HTML（`Content-Type: text/html`）
- 回傳一般 String → JSON 字串

## 首頁

`GET /` 自動產生歡迎頁面，列出所有路由和 curl 範例。

## 搭配模板引擎

```sunny
fn pageShow(id: Int) -> String {
    lit layout = read_file("templates/page.html")
    lit content = render_md(read_file("pages/about.md"))
    out render_template(layout, {
        "title": "About",
        "content": content
    })
}
```

回傳的 HTML 會自動設定正確的 Content-Type。

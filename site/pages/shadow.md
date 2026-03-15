# Shadow 錯誤處理

> Sunny 拒絕隱藏問題。沒有 null，沒有 exception，只有 Shadow。

## 設計哲學

大多數語言用 `null` 和 `try/catch` 處理錯誤，但這些機制容易被忽略。Sunny 的 Shadow 型別讓錯誤成為「一等公民」，強制開發者面對每一個可能出問題的地方。

## 建立 Shadow

```sunny
lit err = Shadow("something went wrong")
print(err)           // Shadow(something went wrong)
print(err.message)   // something went wrong
```

## 自動 Shadow

某些操作會自動回傳 Shadow：

```sunny
// 除以零
lit a = 10 / 0           // Shadow(division by zero)

// List 越界
lit b = [1, 2][99]       // Shadow(index 99 out of bounds)

// Map 找不到 key
lit c = {"a": 1}["z"]    // Shadow(key 'z' not found)
```

## 用 match 處理

`match` 強制你處理 Shadow 的情況：

```sunny
fn userShow(id: Int) -> String | Shadow {
    if id <= 0 {
        out Shadow("invalid user ID")
    }
    out "User #" + to_string(id)
}

lit result = userShow(-1)

match result {
    is String name -> print("Found:", name)
    is Shadow err  -> print("Error:", err.message)
}
```

## Shadow 方法

### .message

取得錯誤訊息字串。

```sunny
lit e = Shadow("not found")
print(e.message)   // not found
```

### .wrap(context)

為錯誤加上情境前綴，方便追蹤錯誤來源。

```sunny
lit e = Shadow("connection refused")
lit wrapped = e.wrap("Database")
print(wrapped.message)   // Database: connection refused
```

### .unwrap_or(default)

如果是 Shadow，回傳預設值。

```sunny
lit e = Shadow("missing")
print(e.unwrap_or("N/A"))   // N/A
```

## 錯誤鏈

Shadow 可以串接 wrap，形成完整的錯誤追蹤鏈：

```sunny
fn dbShow(query: String) -> String | Shadow {
    out Shadow("timeout")
}

fn apiShow(id: Int) -> String | Shadow {
    lit data = dbShow("SELECT *")
    match data {
        is String s -> out s
        is Shadow e -> out e.wrap("apiShow")
    }
}

lit result = apiShow(1)
match result {
    is String s -> print(s)
    is Shadow e -> print(e.message)
}
// apiShow: timeout
```

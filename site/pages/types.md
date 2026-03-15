# 型別系統

Sunny 有 7 種基本型別。

## Int

64 位元整數。

```sunny
lit age = 25
lit negative = -10
lit hex = 255
```

## Float

64 位元浮點數。

```sunny
lit pi = 3.14159
lit temp = -2.5
```

Int 和 Float 可以混合運算，結果為 Float：

```sunny
print(1 + 2.5)    // 3.5
print(10 / 3.0)   // 3.333...
```

## String

UTF-8 字串，支援雙引號和單引號。

```sunny
lit s1 = "hello"
lit s2 = 'world'
lit msg = "Hi, {s1}!"   // 字串內插
```

**方法：** `length` / `trim()` / `upper()` / `lower()` / `contains()` / `starts_with()` / `ends_with()` / `split()`

## Bool

```sunny
lit yes = true
lit no = false
```

## List

有序集合，可包含任意型別。

```sunny
lit nums = [1, 2, 3]
lit mixed = [1, "two", true]
print(nums[0])        // 1
print(nums.length)    // 3
```

**方法：** `push()` / `pop()` / `first()` / `last()` / `contains()` / `reverse()` / `join()`

所有方法都回傳新的 List（不可變操作）。

## Map

鍵值對集合，鍵必須是 String。

```sunny
lit user = {
    "name": "Sunny",
    "age": 25,
    "admin": true
}
print(user["name"])   // Sunny
```

**方法：** `keys()` / `values()` / `has()` / `length()` / `remove()`

## Shadow

錯誤或空值的封裝型別，取代 null 和 exception。

```sunny
lit err = Shadow("something went wrong")
print(err.message)        // something went wrong
print(err.wrap("DB"))     // Shadow(DB: something went wrong)
print(err.unwrap_or(0))   // 0
```

除以零自動回傳 Shadow：

```sunny
lit result = 10 / 0   // Shadow(division by zero)
```

## 聯合型別

函數回傳型別可以用 `|` 組合：

```sunny
fn dataShow(id: Int) -> String | Shadow {
    if id < 0 {
        out Shadow("invalid")
    }
    out "ok"
}
```

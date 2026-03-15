# 教學

從零開始學 Sunny，10 分鐘上手。

## 1. 變數

```sunny
lit name = "Sunny"     // 不可變
glow count = 0         // 可變

count = count + 1      // OK
// name = "other"      // 錯誤！lit 不可修改
```

## 2. 資料型別

```sunny
lit age = 25              // Int
lit pi = 3.14             // Float
lit greeting = "Hello"    // String
lit active = true         // Bool
lit items = [1, 2, 3]     // List
lit user = {"name": "A"}  // Map
lit err = Shadow("oops")  // Shadow
```

## 3. 函數

函數用 `fn` 定義，必須帶 Resource Action 後綴：

```sunny
fn addShow(a: Int, b: Int) -> Int {
    out a + b
}

print(addShow(3, 4))  // 7
```

## 4. 閉包

匿名函數可以省略名稱和型別：

```sunny
lit double = fn(x) { out x * 2 }
print(double(5))  // 10
```

## 5. 條件判斷

```sunny
lit score = 85

if score >= 90 {
    print("A")
} else {
    if score >= 80 {
        print("B")
    } else {
        print("C")
    }
}
```

## 6. 迴圈

```sunny
// for..in 遍歷 List
for item in [1, 2, 3] {
    print(item)
}

// Range 語法
for i in 0..5 {
    print(i)
}

// while 迴圈
glow n = 10
while n > 0 {
    print(n)
    n = n - 1
}
```

## 7. 字串內插

```sunny
lit lang = "Sunny"
lit version = 1
print("Welcome to {lang}!")
```

## 8. 錯誤處理

Sunny 用 Shadow 取代 null 和 exception：

```sunny
fn findShow(id: Int) -> String | Shadow {
    if id < 0 {
        out Shadow("invalid id")
    }
    out "item-" + to_string(id)
}

lit result = findShow(-1)
match result {
    is String s -> print("Found:", s)
    is Shadow e -> print("Error:", e.message)
}
```

## 下一步

- [語法參考](syntax.html) — 完整語法規則
- [型別系統](types.html) — 所有資料型別
- [內建函數](builtins.html) — 可用的內建工具
- [Playground](playground.html) — 線上試玩

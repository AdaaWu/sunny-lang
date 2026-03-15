# 內建函數

## I/O

### print(args...)

輸出到控制台，多個參數用空格分隔。

```sunny
print("hello")           // hello
print("sum:", 1 + 2)     // sum: 3
```

### read_file(path)

讀取檔案內容，回傳 String 或 Shadow。

```sunny
lit content = read_file("data.txt")
```

### write_file(path, content)

寫入檔案，自動建立父目錄。

```sunny
write_file("out/index.html", "<h1>Hello</h1>")
```

## 型別轉換

### to_int(value)

轉換為 Int。支援 String、Float、Bool。

```sunny
to_int("42")      // 42
to_int(3.14)      // 3
to_int(true)      // 1
```

### to_float(value)

轉換為 Float。

```sunny
to_float(42)       // 42.0
to_float("3.14")   // 3.14
```

### to_string(value)

轉換為 String。

```sunny
to_string(42)      // "42"
to_string(true)    // "true"
```

## 工具

### len(value)

取得長度，支援 String、List、Map。

```sunny
len("hello")       // 5
len([1, 2, 3])     // 3
len({"a": 1})      // 1
```

### type_of(value)

回傳型別名稱字串。

```sunny
type_of(42)         // "Int"
type_of("hi")       // "String"
type_of([1, 2])     // "List"
type_of(Shadow("")) // "Shadow"
```

### json_encode(value)

將值轉為 JSON 字串。

```sunny
lit data = {"name": "Sunny", "version": 1}
print(json_encode(data))
// {"name": "Sunny", "version": 1}
```

### time_now()

回傳當前 Unix timestamp（秒）。

```sunny
lit ts = time_now()
print("timestamp:", ts)
```

## 渲染

### render_md(markdown)

將 Markdown 字串轉為 HTML。

```sunny
lit html = render_md("# Hello\n**bold** text")
```

### render_template(template, vars)

將模板中的 `{{ key }}` 替換為 Map 中的值。

```sunny
lit html = render_template(
    "<h1>{{ title }}</h1>",
    {"title": "Welcome"}
)
```

## 併發

### await(ray_handle)

等待 ray 執行緒完成並取得結果。

```sunny
lit handle = ray {
    out 1 + 2
}
lit result = await(handle)
print(result)  // 3
```

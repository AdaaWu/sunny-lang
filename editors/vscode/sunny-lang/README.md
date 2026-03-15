# Sunny Language - VSCode Extension

Syntax highlighting support for the Sunny programming language (`.sunny` files).

## Features

- Syntax highlighting for all Sunny keywords, types, operators, strings, numbers, and comments
- Bracket matching and auto-closing pairs
- Comment toggling (`Cmd+/` for line comments)

## Supported Syntax

| Element   | Examples                                                                        |
| --------- | ------------------------------------------------------------------------------- |
| Keywords  | `lit`, `glow`, `fn`, `if`, `else`, `otherwise`, `match`, `cycle`, `out`, `ray`  |
| Types     | `Int`, `Float`, `String`, `Bool`, `List`, `Map`, `Shadow`                       |
| Logical   | `and`, `or`, `not`, `&&`, `\|\|`, `!`                                           |
| Strings   | `"double"`, `'single'`                                                          |
| Numbers   | `42`, `3.14`                                                                    |
| Comments  | `// single-line`, `/* multi-line */`                                            |
| Booleans  | `true`, `false`                                                                 |
| Operators | `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `->`                                       |

## Installation

### Option 1: Symlink (recommended for development)

**VSCode:**

```bash
ln -s /path/to/sunny-lang/editors/vscode/sunny-lang ~/.vscode/extensions/sunny-lang
```

**Cursor:**

```bash
ln -s /path/to/sunny-lang/editors/vscode/sunny-lang ~/.cursor/extensions/sunny-lang
```

Then reload the editor (`Cmd+Shift+P` → `Reload Window`).

### Option 2: Package as VSIX

```bash
cd editors/vscode/sunny-lang
npx @vscode/vsce package
code --install-extension sunny-lang-0.1.0.vsix
```

---

## Sunny Language - VSCode 擴充套件

為 Sunny 程式語言（`.sunny` 檔案）提供語法高亮支援。

### 功能

- 支援所有 Sunny 關鍵字、型別、運算子、字串、數字及註解的語法高亮
- 括號匹配與自動關閉配對
- 註解快捷鍵（`Cmd+/` 切換單行註解）

### 支援的語法

| 語法元素 | 範例                                                                           |
| -------- | ------------------------------------------------------------------------------ |
| 關鍵字   | `lit`, `glow`, `fn`, `if`, `else`, `otherwise`, `match`, `cycle`, `out`, `ray` |
| 型別     | `Int`, `Float`, `String`, `Bool`, `List`, `Map`, `Shadow`                      |
| 邏輯運算 | `and`, `or`, `not`, `&&`, `\|\|`, `!`                                          |
| 字串     | `"雙引號"`, `'單引號'`                                                         |
| 數字     | `42`, `3.14`                                                                   |
| 註解     | `// 單行註解`, `/* 多行註解 */`                                                |
| 布林值   | `true`, `false`                                                                |
| 運算子   | `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `->`                                      |

### 安裝方式

#### 方式一：Symlink（推薦開發用）

**VSCode：**

```bash
ln -s /path/to/sunny-lang/editors/vscode/sunny-lang ~/.vscode/extensions/sunny-lang
```

**Cursor：**

```bash
ln -s /path/to/sunny-lang/editors/vscode/sunny-lang ~/.cursor/extensions/sunny-lang
```

安裝後重新載入編輯器（`Cmd+Shift+P` → 輸入 `Reload Window` → Enter）。

#### 方式二：打包成 VSIX

```bash
cd editors/vscode/sunny-lang
npx @vscode/vsce package
code --install-extension sunny-lang-0.1.0.vsix
```

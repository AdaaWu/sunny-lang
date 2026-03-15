const EXAMPLES = {
  "hello": {
    name: "Hello World",
    code: `// Sunny Lang - Hello World!
lit name = "Sunny"
print("Hello, {name}!")
print("Welcome to the sunshine!")`,
  },
  "variables": {
    name: "Variables",
    code: `// lit = immutable, glow = mutable
lit pi = 3.14
glow count = 0

count = count + 1
count = count + 1

print("pi:", pi)
print("count:", count)`,
  },
  "functions": {
    name: "Functions",
    code: `// Functions use Resource Action suffixes
fn addShow(a: Int, b: Int) -> Int {
    out a + b
}

fn greetIndex() -> String {
    out "Hello from Sunny!"
}

print(addShow(3, 7))
print(greetIndex())`,
  },
  "closures": {
    name: "Closures",
    code: `// Anonymous functions (closures)
lit double = fn(x) { out x * 2 }
lit add = fn(a, b) { out a + b }

print("double(5) =", double(5))
print("add(3, 4) =", add(3, 4))

// Pass closures as arguments
fn applyShow(f: Int, x: Int) -> Int {
    out f
}
lit triple = fn(x) { out x * 3 }
print("triple(4) =", triple(4))`,
  },
  "lists": {
    name: "Lists & Loops",
    code: `// List operations
lit fruits = ["apple", "banana", "cherry"]
print("fruits:", fruits)
print("length:", fruits.length)
print("first:", fruits.first())

// for..in loop
for fruit in fruits {
    print("I like", fruit)
}

// Range syntax
print("\\nCounting:")
for i in 0..5 {
    print(i)
}`,
  },
  "maps": {
    name: "Maps",
    code: `// Map (key-value pairs)
lit user = {
    "name": "Sunny",
    "role": "admin",
    "level": 42
}

print("name:", user["name"])
print("keys:", user.keys().join(", "))
print("has role?", user.has("role"))

// Immutable operations
lit updated = user.remove("level")
print("after remove:", updated)`,
  },
  "shadow": {
    name: "Shadow (Error Handling)",
    code: `// Shadow replaces null/exceptions
fn findShow(id: Int) -> String | Shadow {
    if id < 0 {
        out Shadow("ID must be positive")
    }
    if id == 0 {
        out Shadow("ID cannot be zero")
    }
    out "Found item #" + to_string(id)
}

// Force handling with match
lit result = findShow(-1)
match result {
    is String val -> print("OK:", val)
    is Shadow err -> print("Error:", err.message)
}

lit result2 = findShow(42)
match result2 {
    is String val -> print("OK:", val)
    is Shadow err -> print("Error:", err.message)
}

// Division by zero returns Shadow
lit div = 10 / 0
print("10 / 0 =", div)`,
  },
  "while": {
    name: "While Loop",
    code: `// While loop with glow (mutable) variable
glow sum = 0
glow i = 1

while i <= 10 {
    sum = sum + i
    i = i + 1
}

print("Sum of 1..10 =", sum)

// FizzBuzz
glow n = 1
while n <= 20 {
    if n % 15 == 0 {
        print("FizzBuzz")
    } else {
        if n % 3 == 0 {
            print("Fizz")
        } else {
            if n % 5 == 0 {
                print("Buzz")
            } else {
                print(n)
            }
        }
    }
    n = n + 1
}`,
  },
  "strings": {
    name: "String Methods",
    code: `// String interpolation
lit lang = "Sunny"
print("Welcome to {lang}!")

// String methods
lit msg = "  Hello, World!  "
print("trim:", msg.trim())
print("upper:", msg.trim().upper())
print("lower:", msg.trim().lower())
print("contains 'World':", msg.contains("World"))
print("starts_with '  H':", msg.starts_with("  H"))

// Split & Join
lit csv = "apple,banana,cherry"
lit parts = csv.split(",")
print("split:", parts)
print("join:", parts.join(" | "))`,
  },
  "builtins": {
    name: "Built-in Functions",
    code: `// Type checking
print("type_of(42):", type_of(42))
print("type_of(3.14):", type_of(3.14))
print("type_of(\\"hi\\"):", type_of("hi"))
print("type_of([1,2]):", type_of([1, 2]))
print("type_of(true):", type_of(true))

// Type conversion
print("to_int(\\"42\\"):", to_int("42"))
print("to_float(42):", to_float(42))
print("to_string(42):", to_string(42))

// Length
print("len(\\"hello\\"):", len("hello"))
print("len([1,2,3]):", len([1, 2, 3]))

// JSON encode
lit data = {"name": "Sunny", "version": 1}
print("json:", json_encode(data))`,
  },
};

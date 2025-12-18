# derive-wizard

A Rust procedural macro that automatically generates interactive CLI wizards from struct definitions using [requestty](https://crates.io/crates/requestty).

## Features

### Supported Question Types

The `#[derive(Wizard)]` macro supports all 11 requestty question types:

| Rust Type | Default Question Type | Override Options | Returns |
|-----------|----------------------|------------------|---------|
| `String` | `input` | `#[mask]` for password, `#[editor]` for text editor | `String` |
| `bool` | `confirm` | - | `bool` |
| `i8`, `i16`, `i32`, `i64`, `isize` | `int` | - | `i64` (cast to type) |
| `u8`, `u16`, `u32`, `u64`, `usize` | `int` | - | `i64` (cast to type) |
| `f32`, `f64` | `float` | - | `f64` (cast to type) |
| `ListItem` | `select` | - | `ListItem` |
| `ExpandItem` | `expand` | - | `ExpandItem` |
| `Vec<ListItem>` | `multi_select` | - | `Vec<ListItem>` |

### Question Type Details

1. **input** - Basic text input prompt (default for String)
2. **password** - Hidden text input (use `#[mask]` on String fields)
3. **editor** - Opens text editor for longer input (use `#[editor]` on String fields)
4. **confirm** - Yes/No confirmation prompt (default for bool)
5. **int** - Integer input (default for integer types)
6. **float** - Floating point input (default for float types)
7. **select** - Single selection from a list (default for ListItem)
8. **expand** - Single selection with keyboard shortcuts (default for ExpandItem)
9. **multi_select** - Multiple selection from a list (default for Vec<ListItem>)

Note: The following question types are available in requestty but not currently exposed through attributes:

- **raw_select** - Single selection with index-based input
- **order_select** - Reorder items in a list

## Usage

### Basic Example

```rust
use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
struct Config {
    #[prompt("Enter the server address:")]
    server: String,  // Uses 'input' question type by default

    #[prompt("Enter the port number:")]
    port: u16,  // Uses 'int' question type by default
    
    #[prompt("Enable logging?")]
    logging: bool,  // Uses 'confirm' question type by default
}

fn main() {
    let config = Config::wizard();
    println!("Config: {config:#?}");
}
```

### Password Fields with `#[mask]`

For password inputs, use the convenient `#[mask]` attribute to hide user input:

```rust
use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
struct LoginForm {
    #[prompt("Enter your username:")]
    username: String,

    #[prompt("Enter your password:")]
    #[mask]
    password: String,  // Input will be hidden
}
```

### Long Text with `#[editor]`

For longer text input, use the `#[editor]` attribute to open the user's preferred text editor:

```rust
use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
struct Article {
    #[prompt("Enter the title:")]
    title: String,

    #[prompt("Write the article content:")]
    #[editor]
    content: String,  // Opens text editor (vim, nano, etc.)
}
```

### All Supported Types

```rust
use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
struct ComprehensiveExample {
    // String-based prompts
    #[prompt("Basic input:")]
    basic_input: String,
    
    #[prompt("Password:")]
    #[mask]
    password: String,
    
    #[prompt("Long description:")]
    #[editor]
    description: String,
    
    // Boolean
    #[prompt("Agree to terms?")]
    agree: bool,
    
    // Integers
    #[prompt("Your age (i32):")]
    age: i32,
    
    #[prompt("Count (u64):")]
    count: u64,
    
    // Floats
    #[prompt("Height in meters:")]
    height: f64,
    
    #[prompt("Weight in kg:")]
    weight: f32,
}
```

## Attributes

- `#[prompt("message")]` - **Required**. The message to display to the user
- `#[mask]` - **Optional**. For String fields: enables password input (hidden text)
- `#[editor]` - **Optional**. For String fields: opens text editor for longer input

**Note**: `#[mask]` and `#[editor]` are mutually exclusive and cannot be used on the same field.

### Examples

```rust
// Password field
#[prompt("Password:")]
#[mask]
password: String,

// Long text with editor
#[prompt("Description:")]
#[editor]
description: String,
```

## Requirements

```toml
[dependencies]
derive-wizard = "0.1.0"
```

## License

MIT OR Apache-2.0

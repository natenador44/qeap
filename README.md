# QEAP

**Q**uick and **E**asy **A**pplication **P**ersistence

A Rust library that provides a pluggable interface for persisting application data with minimal boilerplate. QEAP acts as an abstraction layer (similar to how `serde` relates to `serde_json`), allowing you to choose your persistence mechanism while keeping your application code clean and simple.

## Features

- **Pluggable Persistence**: Choose any persistence backend (files, databases, cloud storage, etc.)
- **Zero-boilerplate**: Simply derive `Qeap` on your types, specify a persistence mechanism, and meet its requirements.
- **Automatic Load/Save**: The `scoped` macro handles the complete lifecycle automatically
- **Interior Mutability Support**: Built-in implementations for `RefCell`, `Mutex`, and other wrapper types
- **Type-safe**: Leverages Rust's type system for reliable operations
- **Flexible**: Works with any type that implements `Qeaper`

## Architecture

QEAP provides two core traits:

- **`Qeap`**: The main trait that your data types implement (usually via derive macro)
- **`Qeaper`**: The trait that defines how data is stored and loaded

This separation allows you to:
- Use different storage backends for different types
- Switch persistence mechanisms without changing your data structures
- Create custom persistence implementations for specific needs

## Installation

Add `qeap` to your project along with a persistence implementation:

```toml
[dependencies]
qeap = "0.1"
qeap-file = "0.1"  # For file-based persistence
serde = { version = "1.0", features = ["derive"] }
```

## Basic Usage

### 1. Define Your Data Structure

Derive `Qeap` and specify a persistence mechanism:

```rust
use qeap::Qeap;
use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize, Qeap)]
#[qeap(with = qeap_file::JsonFile::new("app_data"))]
struct AppConfig {
    port: u16,
    max_connections: u32,
    api_key: Option<String>,
}
```

Note: For persistence mechanisms that are expensive to create, you can use something like `LazyLock` from the standard library.

```rust
use qeap::Qeap;
use serde::{Serialize, Deserialize};
use std::sync::LazyLock;

static DATABASE: LazyLock<AppConfigDatabase> = LazyLock::new(|| {
    let url = std::env::var("APP_CONFIG_DB_URL").expect("APP_CONFIG_DB_URL environment variable is required");
    AppConfigDatabase::connect(url).expect("connection successful")
})

#[derive(Default, Debug, Serialize, Deserialize, Qeap)]
#[qeap(with = &*DATABASE)]
struct AppConfig {
    port: u16,
    max_connections: u32,
    api_key: Option<String>,
}
```

In the future there may be a `try_with` to handle persistence creation errors.

### 2. Load and Save Data

The `Qeap` trait provides methods to interact with your data:

```rust
fn main() -> Result<(), qeap::Error> {
    // Load existing data or create with defaults
    let mut config = AppConfig::load()?;

    // Modify your data
    config.port = 8080;
    config.max_connections = 100;

    // Save changes
    config.save()?;

    Ok(())
}
```

### 3. Automatic Scoped Persistence

Use the `scoped` macro to automatically handle load/save cycles:

```rust
#[qeap::scoped]
fn main(config: &mut AppConfig) -> Result<(), qeap::Error> {
    println!("Server running on port {}", config.port);
    config.port = 9000;  // Changes are automatically saved on exit
    Ok(())
}
```

The `scoped` macro expands your function to:
1. Load data before the function runs
2. Execute your function with references to the loaded data
3. Save data after the function completes

#### Using `scoped`
The main focus of `scoped` is for use by the `main` function of your application. However, it does not need to be used there. It can be used on any function.

```rust
#[qeap::scoped]
fn update_port(app_data: &mut AppConfig) {
    app_data.port = 8080;
}
```

You would call this like so.
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    update_port()?;
    Ok(())
}
```

While techniclly possible, I personally am not a fan of this for several reasons.
1. The call signature is different than the defined signature and can be confusing.
2. You can't specify additional, non-`Qeap` function parameters. It's all or none (at least for now - not sure how that would look but it's something I'm going to go for eventually if the desire is there).

This is why I recommend using this for global application data that you want to load when your program first starts up.

However, for those who are fine with this, `scoped` has a few modes you can specify that modify how it works and what the scoped function returns.

##### Scoped Modes
You do have some control over how the `scoped` function is generated, via "modes" you can specify which change what the function returns.
###### nested
This is the default (used with a plain `#[qeap::scoped]`). This takes whatever is returned from your function and wraps it in a `Result<T, qeap::error::Error>`.
If `T` is `u16`, then the scoped function will return `Result<u16, qeap::error::Error>`. If your function returns `Result<u16, MyError>`, then
the scoped function will return `Result<Result<u16, MyError>, qeap::error::Error>`.
The advantage of this is you can return whatever you want from the function, and don't have to worry about any special rules. The downside is nested types aren't
ergonomic to deal with.
If you want to be explicit, you can write `#[qeap::scoped(nested)]`.

###### flatten
This flattens your type into a top-level `Result`, regardless of the return type.
For a return type of `u16`, the scoped function will return `Result<u16, qeap::error::FlattenedError<Infallible>>`.
For a return type of `Result<u16, MyError>`, the scoped function will return `Result<u16, qeap::error::FlattenedError<MyError>>`.
You can then match on the error to either get the `qeap::error::Error` or your error.
```rust
#[qeap::scoped(flatten)]
fn do_something(app_data: &AppData) -> Result<u16, MyError> {
    match app_data.port {
        8080 => Err(MyError::InvalidPort),
        other => Ok(other)
    }
}

fn main() {
    match do_something() {
        Ok(port) => println!("{port}"),
        Err(FlattenedError::Qeap(e)) => println!("failed to persist data: {e}"),
        Err(FlattenedError::User(MyError::InvalidPort)) => println!("app data contained invalid port"),
    }
}
```

This has a similar advantage to `nested`, but is arguably more ergonomic if you need to use the result. It's mostly up to your own personal preference.

###### absorb
This will work on any function with a return type that matches this requirement: `Result<T, E> where E: From<qeap::error::Error>`.
This option gives you a more stable signature, since `scoped` doesn't change it, but forces the return type to be a `Result`.

```rust
enum MyError {
    InvalidPort,
    Qeap(qeap::error::Error)
}

impl From<qeap::error::Error> for MyError {
    fn from(e: qeap::error::Error) -> Self {
        Self::Qeap(e)
    }
}

#[qeap::scoped(absorb)]
fn do_something(app_data: &AppData) -> Result<u16, MyError> {
    match app_data.port {
        8080 => Err(MyError::InvalidPort),
        other => Ok(other)
    }
}

fn main() {
    match do_something() {
        Ok(port) => println!("{port}"),
        Err(MyError::Qeap(e)) => println!("failed to persist data: {e}"),
        Err(MyError::InvalidPort) => println!("app data contained invalid port"),
    }
}
```

###### expect
This option forces all `Qeap` operations to `expect()` success, taking their failure out of the equation from a scoped return type perspective.
This is the cleanest as far as handling the result of a scoped function, with the downside of `Qeap` crashing your program if it fails.

```rust
enum MyError {
    InvalidPort,
}


#[qeap::scoped(expect)]
fn do_something(app_data: &AppData) -> Result<u16, MyError> {
    match app_data.port {
        8080 => Err(MyError::InvalidPort),
        other => Ok(other)
    }
}

fn main() {
    match do_something() {
        Ok(port) => println!("{port}"),
        Err(MyError::InvalidPort) => println!("app data contained invalid port"),
    }
}
```

If panic handling is introduced to `scoped` in the future, that may make this a bit better.. but I'm still on the fence on panic handling.

## Persistence Implementations

QEAP doesn't provide persistence implementations directly. Instead, use companion crates:

### qeap-file

For file-based persistence with various formats:

```rust
use qeap_file::{JsonFile, TomlFile, YamlFile};

#[derive(Default, Serialize, Deserialize, Qeap)]
#[qeap(with = TomlFile::new("config_dir"))]
struct Config {
    theme: String,
    font_size: u8,
}
```

Available formats:
- `JsonFile::new(dir)` - JSON format
- `TomlFile::new(dir)` - TOML format
- `YamlFile::new(dir)` - YAML format

Files are automatically named based on your struct name (e.g., `Config` → `config.toml`).

### Custom Persistence

Implement `Qeaper` for custom storage:

```rust
use qeap::Qeaper;

struct DatabaseBackend {
    connection_string: String,
}

impl Qeaper for DatabaseBackend {
    type Error = MyDatabaseError;

    fn load<T: DeserializeOwned>(&self) -> Result<T, Self::Error> {
        // Load from database
    }

    fn save<T: Serialize>(&self, data: &T) -> Result<(), Self::Error> {
        // Save to database
    }
}
```

## Scoped Supported Types
QEAP supports `&T` and `&mut T` parameters, as well as wrapper types like `Arc`, `Rc`, `Mutex`, and `RefCell` as parameters in `scoped` functions.

```rust
#[derive(Default, Serialize, Deserialize, Qeap)]
#[qeap(with = JsonFile::new("data"))]
struct AppState {
    counter: u32,
    items: Vec<String>,
}

#[qeap::scoped]
fn main(state: Arc<Mutex<AppState>>) -> Result<(), qeap::Error> {
    let mut state = state.lock().unwrap();
    state.counter += 1;
    state.items.push("item".to_string());
    Ok(())
}
```

Supported wrapper types for `scoped` parameters:
- `Arc<T>`
- `RwLock<T>`
- `Rc<T>`
- `Mutex<T>`
- `RefCell<T>`
- Combinations like `Arc<Mutex<T>>`

You can also implement your own if `qeap` doesn't automatically implement it for you.
```rust
struct MyWrapperType<T>(T);

impl<T> qeap::Qeap for MyWrapperType<T> {
    fn load() -> QeapResult<Self>
    where
        Self: Sized
    {
        Ok(MyWrapperType(T::load()?))
    }

    fn save(&self) -> QeapResult<()> {
        self.0.save()
    }
}
```

**Note**: Whether wrapper types can be used *within* your data structures (e.g., `struct AppState { counter: RefCell<u32> }`) depends on your persistence mechanism's serialization support.
For example, persistence mechanisms that utilize `serde` will support these with the appropriate `serde` feature flags.

## Advanced Examples

### Multiple Data Types

```rust
#[derive(Default, Serialize, Deserialize, Qeap)]
#[qeap(with = TomlFile::new("app_config"))]
struct Config {
    port: u16,
    host: String,
}

#[derive(Default, Serialize, Deserialize, Qeap)]
#[qeap(with = JsonFile::new("user_data"))]
struct UserPreferences {
    theme: String,
    notifications: bool,
}

#[qeap::scoped]
fn main(
    config: &Config,
    prefs: &mut UserPreferences,
) -> Result<(), qeap::Error> {
    println!("Server: {}:{}", config.host, config.port);
    prefs.notifications = true;  // Saved automatically
    Ok(())
}
```

## Error Handling

QEAP provides a unified error type that persistence mechanisms can integrate with:

```rust
use qeap::Error;

fn process_data() -> Result<(), Error> {
    let data = MyData::load()?;  // Returns qeap::Error
    data.save()?;
    Ok(())
}
```

## Limitations and Considerations

### Current Limitations

- **No panic handling**: If code panics within a `scoped` function, data is not saved
- **No signal handling**: Interrupts (Ctrl+C) or kills won't trigger saves
- **Synchronous only**: Async support planned for future releases
- **Performance**: Not optimized for high-frequency saves or performance-critical applications

### Best Practices

- Use QEAP for application configuration, user preferences, and caching
- Avoid using QEAP in hot loops or performance-critical paths
- For complex data relationships, consider a proper database
- Implement proper error handling in your application logic

## Roadmap

- **Async support**: Async versions of `Qeap` trait methods
- **Signal handling**: Optional feature to save on interrupts
- **Optimized saves**: Only save when data actually changes
- **Transaction support**: Atomic save operations for consistency
- **Additional backends**: Built-in support for more storage types

## Use Cases

Perfect for:
- Application configuration files
- User preferences and settings
- Local cache data
- Development and prototyping
- Small to medium-sized desktop applications

Not ideal for:
- High-performance applications with frequent writes
- Complex relational data
- Distributed systems requiring consistency
- Real-time applications

## Contributing

Contributions are welcome! Whether it's:
- New persistence mechanism implementations
- Bug fixes and improvements
- Documentation enhancements
- Example applications

## License

[Include your license information here]

## Related Crates

- `qeap-file` - File-based persistence with multiple format support
- `qeap-macro` - Procedural macros (re-exported by `qeap`)

---

For more examples and detailed API documentation, visit [docs.rs/qeap](https://docs.rs/qeap).

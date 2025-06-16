# QEAP - Quick and Easy Application Persistence

QEAP is a Rust library that eliminates the boilerplate typically required for storing and loading application data to/from JSON files on disk. With just a few annotations, you can add persistent storage capabilities to any serializable Rust struct.

## Key Features

- Zero-boilerplate persistence: Simply derive Qeap on your types and specify a storage directory
- Automatic file management: Creates directories and JSON files automatically
- Default initialization: If no saved data exists, loads using the struct's Default implementation
- Type-safe operations: Leverages Rust's type system and serde for reliable serialization/deserialization
- Comprehensive error handling: Detailed error types for initialization, file I/O, and JSON parsing operations
- Scoped operations: Advanced macro for automatic load/save cycles within function scopes

## How It Works

QEAP provides a derive macro that automatically implements persistence methods on your structs:

-  load(): Loads data from disk, creating a default instance if no file exists
-  save(&self): Saves the current instance to disk
-  file_path(): Returns the full path where the data is stored

The library automatically:
1. Creates the specified directory structure if it doesn't exist
2. Generates JSON filenames based on your struct's name (e.g., Config → Config.json)
3. Handles the complete save/load lifecycle with proper error handling via the `scoped` macro

## Architecture

The project consists of two main crates:

- qeap: The main library providing the Qeap trait, load/save functionality, and error types
- qeap_macro: Procedural macros for the #[derive(Qeap)] and #[qeap::scoped]

## Use Cases

Perfect for:
- Application configuration files
- User preferences and settings
- Cache data that needs persistence
- Simple data storage without database complexity
- Rapid prototyping with persistent state

This library is ideal for developers who want persistent storage with minimal setup - just add the derive macro and you're ready to go!

## How To Use
Add `qeap` and `serde` to your project.
```sh
cargo add qeap
cargo add serde -F derive
```

Derive `qeap::Qeap`, `serde::Serialize`, and `serde::Deserialize` on the type(s) you wish to manage, then specify the directory you wish the store your data in via the `qeap` attribute macro.
```rust
use qeap::Qeap;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Qeap)]
#[qeap(dir = app_dir())] // `dir` is required
struct Config {
    timeout_seconds: u16,
    port: u16,
    log_location: PathBuf,
}

fn app_dir() -> PathBuf {
    std::env::home_dir()
        .map(|d| d.join("my_app"))
        .expect("home directory exists")
}
```

Deriving `qeap::Qeap` adds some methods to your type, `load()` and `save(&self)` (among others). You can use these to initialize an instance of your type and save updates respectively.

```rust
fn main() -> Result<(), Box<dyn std::error::Error> {
    let mut config = Config::load()?;

    config.port = 3000;
    config.save()?;
}
```

### `qeap::scoped`

`qeap::scoped` automates things even further. Annotate a function call with `qeap::scoped`, then specify immutable or mutable references to types that derive `qeap::Qeap` as the function arguments. The function is restructured to automatically load the specified data, pass it to your function as requested,
then save the data to disk.
The function must have a return type of `Result<T, E> where E: From<qeap::error::Error>`.

```rust
#[qeap::scoped]
fn update_port(config: &mut Config) -> qeap::QeapResult<()> {
    config.port = 8080;
}
```

The `update_port` function is expanded into this.
```rust
fn update_port() -> qeap::QeapResult<()> {
    fn update_port_inner(config: &mut Config) -> qeap::QeapResult<()> {
        config.port = 8080;
        Ok(())
    }
    let mut config: Config = qeap::Qeap::load()?;
    let result = update_port_inner(&mut config);
    qeap::Qeap::save(&config)?;
    return result;
}
```

In order to call this function, you would call it without passing arguments.

```rust
fn main() -> qeap::QeapResult<()> {
    update_port()?;
}
```

Calling a function declared with arguments and not passing any into it at the call site might seem a bit unsavory to a lot of people, and I would agree.
For those that don't care, you have the option to use this macro as described above. However, the `main` purpose of this macro is to annotate
the `main` method of your application, like so.

```rust
#[qeap::scoped]
fn main(config: &mut Config) -> qeap::QeapResult<()> {
    // do stuff with `config`
}
```

This generates a `main` method for your application that automatically loads the data you need at the start, then saves it right before the application exits.

#### Panic inside `qeap::scoped` functions
There is none at the moment. If you annotate a function with `qeap::scoped` and code within that function panics, `qeap::scoped` does not handle that, and will not save your data. This may be supported in the future (as an optional feature). It's important to remember that even if this becomes a feature, it won't do diddly-squat if your executable is configured to abort on panic. This is why I am undecided at the moment.

#### Signals
`qeap::scoped` does not handle signals sent to your program (e.g. interrupt or kill). So if your function is running and you press Ctrl-c, your data won't be saved.

## Future Features

### Different File Formats
The user of the library should be able to choose from different file formats (e.g. JSON, TOML, YAML).

### Signal Handling for `qeap::scoped`
At the moment, if a function annotated with `qeap::scoped` is interrupted or killed, the save will not happen. Ideally there'd be an optional feature that can be turned on that catches these types of signals and handles them gracefully (i.e. saving data managed by QEAP).

### Async Support
Ideally, `qeap::scoped` could be used with `async` functions. This will require an `async` version of the `Qeap` trait or its methods, which does not exist in this version. It would also require either choosing an async I/O implementation (like tokio), forcing users of the library to use that implementation, or abstracting it away and letting the user choose one. This is all work that will be done, but only in later versions.
It would also be important that `qeap::scoped` is compatible with other async main method macro alterations, like `tokio::main`.

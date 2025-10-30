mod file;

pub use file::FilePersist;

#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "json")]
pub type JsonFile<T> = FilePersist<json::Json<T>>;

#[cfg(feature = "toml")]
pub mod toml;
#[cfg(feature = "toml")]
pub type TomlFile<T> = FilePersist<toml::Toml<T>>;

mod loader;
mod types;

pub use loader::ConfigLoader;
pub use types::{Config, InputData};

// Re-export for tests
#[cfg(test)]
pub use types::ApiConfig;

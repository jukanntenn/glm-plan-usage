mod loader;
mod types;

pub use loader::ConfigLoader;
#[allow(unused_imports)]
pub use types::{
    ApiConfig, CacheConfig, Config, DisplayMode, IconConfig, InputData, SegmentConfig, StyleConfig,
};

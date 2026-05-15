//! Configuration loading and types for the GLM plan usage plugin.

mod loader;
mod migration;
mod template;
mod types;

#[doc(inline)]
pub use loader::ConfigLoader;
#[doc(inline)]
#[allow(unused_imports, reason = "re-exported for public API")]
pub use migration::MigrationResult;
#[doc(inline)]
#[allow(unused_imports, reason = "re-exported for public API")]
pub use types::{
    ApiConfig, CacheConfig, Config, DisplayMode, IconConfig, InputData, MultiplierConfig,
    PromoConfig, SegmentConfig, StyleConfig, DEFAULT_SEPARATOR,
};

pub use self::platform::*;

#[allow(clippy::module_inception)]
#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod platform;

#[allow(clippy::module_inception)]
#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod platform;

#[allow(clippy::module_inception)]
#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod platform;

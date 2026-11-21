pub mod game;
#[cfg(not(target_arch = "wasm32"))]
pub mod server;
#[cfg(target_arch = "wasm32")]
pub mod wasm_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod ws;

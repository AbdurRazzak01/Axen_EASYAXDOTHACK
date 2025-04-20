#![cfg_attr(not(feature = "std"), no_std)]

// Import external crates
extern crate alloc;

// Standard stuff only available if "std" feature
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// Rest of your imports here
pub mod apis;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod configs;
pub mod genesis_config_presets;
pub mod opaque;

// Type aliases, runtime version etc follow...

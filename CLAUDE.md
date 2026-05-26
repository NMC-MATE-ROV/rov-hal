# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

**rov-pi** is a small Rust service that exposes an HTTP API and WebSocket interface to control Raspberry Pi GPIO devices (LED PWM, servos, ESC) using axum + rppal.

## Commands
- Local build: `cargo build`
- Release: `cargo build --release`
- Cross-compile for Pi: `cargo build --target aarch64-unknown-linux-gnu --release`
- Run: `cargo run` (requires GPIO hardware access)
- Single test: `cargo test <name> -- --exact`
- Format: `cargo fmt`
- Lint: `cargo clippy`
- Deploy: `./deploy.sh`

## Architecture

**Entry point** (`src/main.rs`): tokio runtime, axum server on 0.0.0.0:3000, routes for `/ws` and `/api/device/{device}/{cmd}`

**HAL layer** (`src/hal/`): device drivers under `src/hal/devices/*.rs` exported via `mod.rs`. Factory function `create_gpio_devices()` in `device.rs` maps `RawDevice` configs to concrete drivers.

**Device registry**: `DevicesRegistry = Arc<Mutex<HashMap<String, SharedDevice>>>` where `SharedDevice = Arc<Mutex<Box<dyn Device>>>`

**Device types**: `led` (PWM), `servo` (PWM), `blue_esc` (ESC motor), `digital_input` (passive), `basic_pwm` (generic PWM), `system` (always exists, configurable)

## Command Pattern
Commands are dispatched by name with optional JSON params. Handlers return `serde_json::Value` on success or error string.

## Key Conventions
- HAL pattern: add drivers under `src/hal/devices`, export in `mod.rs`
- Shared handles: `Arc<Mutex<Box<dyn Device>>>`
- System device: always exists, optionally configured via `devices.json` settings
- LED PWM params: `{ frequency, duty_cycle, enable? }` with duty_cycle in 0.0..1.0
- GPIO numbering: BCM pin numbers directly

## Development Notes
- Add new device types: implement `Device` trait, register in `mod.rs`
- Hardware tests require physical GPIO access
- Cross-compilation needs `aarch64-linux-gnu-gcc` linker

## First Files to Inspect
1. `src/main.rs` — routing, server
2. `src/hal/devices/device.rs` — schema, trait, factory
3. `src/hal/devices/led.rs` — LED + PWM
4. `src/hal/devices/servo.rs` — servo
5. `src/hal/devices/blue_esc.rs` — ESC
6. `src/system.rs` — system device (always exists, configurable)
7. `.cargo/config.toml` / `deploy.sh` — cross-compile

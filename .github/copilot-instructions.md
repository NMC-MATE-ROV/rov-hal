# Copilot instructions for rov-pi

Summary
- Small Rust service that exposes an HTTP API and WebSocket interface to control Raspberry Pi GPIO devices (LED PWM, servos) using axum + rppal. Uses tokio runtime. Cross-compile + rsync deployment helper provided.

Build, test, and lint commands
- Local build (native):
  - cargo build
- Release build (native):
  - cargo build --release
- Cross-compile for Raspberry Pi (used by repo):
  - cargo build --target aarch64-unknown-linux-gnu --release
- Run (native, requires access to GPIO/hardware):
  - cargo run
- Run release binary locally:
  - cargo run --release
- Single test (run one test by name):
  - cargo test <test-name> -- --exact
- Full test suite:
  - cargo test
- Format:
  - cargo fmt
- Lint (Clippy):
  - rustup component add clippy && cargo clippy

Deployment helper
- deploy.sh cross-compiles to aarch64-unknown-linux-gnu and rsyncs the release binary to rov@rov-pi:/home/rov/bin/rov-pi.
  - Run: ./deploy.sh
- See .cargo/config.toml for the cross-linker setting (target.aarch64-unknown-linux-gnu.linker).

High-level architecture
- Entrypoint: src/main.rs
  - Starts a tokio runtime and an axum HTTP server. The code binds to 0.0.0.0:3000 by default.
  - Exposes WebSocket at /ws and REST POST endpoints at /api/device/:device/:cmd.
  - Reads device configuration from: /home/rov/.config/devices.json (JSON array of RawDevice objects).
- HAL layer: src/hal/
  - Device drivers live under src/hal/devices/*.rs and are exported via src/hal/devices/mod.rs.
  - Device creation is handled by create_gpio_devices(...) which maps RawDevice entries to concrete drivers and registers GPIO pins.
- Runtime types & flow
  - RawDevice (serde-deserializable) = { id: String, device_type: String, pin: u8, is_input: Option<bool> }
  - Device trait: fn handle(&mut self, cmd: &str, params: &serde_json::Value) -> Result<serde_json::Value, String>
  - SharedDevice = Arc<Mutex<Box<dyn Device + Send + Sync>>>; DevicesRegistry = Arc<Mutex<HashMap<String, SharedDevice>>>
  - create_gpio_devices currently supports "led" and "servo" device_type; unknown types return an error.

Key conventions and patterns
- HAL module pattern: add new drivers under src/hal/devices and export them in src/hal/devices/mod.rs.
- Shared device handles: use Arc<Mutex<Box<dyn Device>>> so handlers can lock, mutate, and call Device::handle.
- Device commands: commands are dispatched by name (string) with JSON params. Handlers return serde_json::Value on success or an error string.
- LED behavior (example): led::handle supports "pwm" with params { "frequency": f64, "duty_cycle": f64, "enable": bool? } where duty_cycle is 0.0..1.0.
- WebSocket messages: GenericWsMessage { device: String, cmd: String, params: Option<Value> } — same dispatch model as REST endpoint.
- Error responses: REST and WS responses use JSON { "error": "..." } for errors; successful REST responses return JSON bodies.
- GPIO numbering: pins are used directly as BCM numbers in create_gpio_devices (check device config for correct numbering).
- Concurrency: devices registry is an Arc<Mutex<...>> and individual devices are Arc<Mutex<Box<dyn Device>>>; hold locks only as long as necessary.

Files to inspect first
- src/main.rs (routing, server runtime, devices registry)
- src/hal/devices/device.rs (RawDevice schema, Device trait, create_gpio_devices)
- src/hal/devices/led.rs (LED implementation and PWM payload expectations)
- src/hal/devices/servo.rs (servo driver)
- .cargo/config.toml and deploy.sh (cross-compile + deploy flow)

AI assistant / other config checks
- This file (copilot instructions) is present and updated. A CLAUDE.md exists in the repo root for additional context.

Notes for future Copilot sessions
- For changes that touch hardware behavior, review src/hal/devices/* and devices.json before editing.
- Cross-compilation relies on an aarch64 linker toolchain; ensure the host has aarch64-linux-gnu-gcc (or set a different linker in .cargo/config.toml).
- Running tests or CI that exercise rppal will require hardware or mocks. There are no mocks currently in the repo.

Questions for future sessions
- Is there a hardware emulator or preferred test harness for rppal-based code to reference?

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is a Rust embedded bootloader for the STM32F439 (ARM Cortex-M4). It is a `no_std`/`no_main` project targeting `thumbv7em-none-eabihf`.

The core logic lives in `src/main.rs` and consists of a single unsafe `jump_to_app(app_addr: u32)` function that performs a clean handoff to an application firmware image.

## Build, Flash, and Run

| Task | Command |
|------|---------|
| Build release | `cargo build --release` |
| Flash and run via probe-rs | `cargo run --release` |
| Flash only | `cargo flash --release` |
| Flash/debug via `cargo-embed` | `cargo embed --release` |

The runner is configured in `.cargo/config.toml` as `probe-rs run --chip STM32F439ZITx`.

## Logging

The project uses `defmt` for logging over RTT. The log level is controlled by `DEFMT_LOG` in `.cargo/config.toml` (currently set to `info`). `panic-probe` prints panic messages to the defmt RTT channel.

## Architecture

### Bootloader Handoff (`jump_to_app`)

`src/main.rs:16` contains the critical `jump_to_app` function. It performs the following steps in exact order before transferring control:

1. **Disable global interrupts** via `cortex_m::interrupt::disable()`.
2. **Stop SysTick** and disable its interrupt.
3. **Clear NVIC** by writing `0xFFFFFFFF` to `NVIC_ICER` and `NVIC_ICPR` registers (8 iterations to cover all interrupt lines).
4. **Reset peripherals** via RCC: pulse reset bits on AHB1, AHB2, APB1, APB2 buses (write `0xFFFF_FFFF` then `0`).
5. **Relocate vector table** by writing `app_addr` to `SCB->VTOR`.
6. **Jump** by loading the stack pointer (`MSP`) and reset vector from the application vector table, then executing `msr msp` + `bx` via inline assembly.

### Memory Layout

`memory.x` defines:

| Region | Address | Size |
|--------|---------|------|
| FLASH | `0x0800_0000` | 2048K |
| CCMRAM | `0x1000_0000` | 64K |
| RAM | `0x2000_0000` | 192K |

The stack starts at the top of RAM (`ORIGIN(RAM) + LENGTH(RAM)`).

## Key Files and Configuration

- `.cargo/config.toml` — Target (`thumbv7em-none-eabihf`), linker flags (`link.x`, `defmt.x`), probe-rs runner, `DEFMT_LOG` env var.
- `Embed.toml` — `cargo-embed` configuration for STM32F439ZITx with SWD protocol.
- `build.rs` — Downloads `stm32f439.svd.patched` from `stm32-rs.github.io` at build time if not present.
- `Cargo.toml` — Uses `stm32f4xx-hal` with feature `stm32f439` and `defmt`. Release profile uses `opt-level = "s"`, `lto = true`, `codegen-units = 1`.

## Dependencies

- `cortex-m`, `cortex-m-rt` — Core embedded runtime
- `stm32f4xx-hal` — HAL for STM32F4, feature-gated for `stm32f439`
- `defmt`, `defmt-rtt` — Structured logging over RTT
- `panic-probe` — Panic handler that prints to defmt

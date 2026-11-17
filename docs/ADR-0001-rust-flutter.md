# ADR-0001: Rust Engine and Flutter UI

- **Status:** Accepted
- **Date:** 2026-11-17

## Context
The reference combines Kotlin/JVM core services, JavaFX/TornadoFX desktop UI, Spring Boot services and a separate Kotlin Multiplatform mobile client. The target must ship without a Java/JVM runtime dependency and support 64-bit desktop/mobile targets while preserving behavior.

## Decision
- Rust owns domain logic, URL classification, parsing, host resolution, HTTP/download execution, scheduling, persistence, output naming and platform-independent engine state.
- Flutter owns rendering and interaction, reproducing reference UI and navigation.
- A versioned FFI contract separates Flutter from Rust.
- OS-specific operations (secure storage, file pickers, notifications, drag/drop, background execution and lifecycle) use narrow platform adapters.
- Share the Rust domain engine and compatibility fixtures across desktop/mobile where platform policies allow.

## Consequences
Benefits:
- Native compiled engine with no JVM runtime requirement.
- Explicit ownership and memory safety for concurrent download work.
- Shared UI implementation and testable separation of domain/presentation.

Trade-offs:
- Rust migration is a substantial rewrite; behavior must be captured first.
- Flutter is cross-platform rendered UI, not identical OS-native widgets.
- Mobile background execution, storage and networking differ by OS.
- Android packaging may need JDK/Android SDK at build time; installed apps must not need Java at runtime.
- GPL-3.0 and third-party licensing obligations remain applicable.

# Ferroscope — Implementation Plan

> 🦀💨 See Rust's power in action. Real-time visuals of memory, speed, and zero-cost abstractions. No GC. No compromise.

---

## Overview

Ferroscope is a 100% Rust interactive terminal application (TUI) that visually demonstrates every major capability of the Rust programming language in real-time. It is educational, shareable, and built to run anywhere — including the browser via WebAssembly.

---

## Core Design Philosophy

- **100% Rust** — no JavaScript, no Python scripts, no shell glue
- **Terminal-first** — runs in any terminal with color and Unicode support
- **WASM-ready** — the same codebase compiles to WebAssembly for browser deployment
- **Real data, not fake animations** — benchmarks run live, metrics are real system data
- **Self-describing** — every screen explains what Rust concept it demonstrates and why it matters

---

## Tech Stack

```toml
[dependencies]
# TUI framework
ratatui        = "0.29"       # Terminal UI rendering
crossterm      = "0.28"       # Cross-platform terminal backend

# Async runtime
tokio          = { version = "1", features = ["full"] }

# Concurrency / parallelism
rayon          = "1"          # Data parallelism (work-stealing thread pool)
parking_lot    = "0.12"       # Fast Mutex/RwLock showcase

# System metrics
sysinfo        = "0.33"       # Live CPU/memory/process info

# Benchmarking
criterion      = "0.5"        # Micro-benchmark harness (bench targets)
instant        = "0.1"        # WASM-compatible timing

# Memory tracking
tikv-jemalloc-ctl = "0.5"    # jemalloc stats (native)

# Utilities
crossbeam      = "0.8"        # Channels, scoped threads, atomics
once_cell      = "1"          # Lazy statics
anyhow         = "1"          # Error handling demo substrate
thiserror      = "1"          # Custom error types showcase
rand           = "0.8"        # Randomness for benchmarks/demos

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
# Native-only
sysinfo        = "0.33"

[target.'cfg(target_arch = "wasm32")'.dependencies]
# WASM-only
wasm-bindgen   = "0.2"
web-sys        = { version = "0.3", features = ["Performance", "Window"] }
getrandom      = { version = "0.2", features = ["js"] }
```

---

## Repository Structure

```
ferroscope/
├── src/
│   ├── main.rs                        # Entry point — parse args, init runtime, run app
│   ├── app.rs                         # App struct, global state, event loop
│   ├── events.rs                      # Input event handling (keyboard, mouse, resize)
│   ├── theme.rs                       # Color palette, styles, Unicode symbols
│   │
│   ├── ui/
│   │   ├── mod.rs                     # Root render dispatch
│   │   ├── layout.rs                  # Responsive layout calculation
│   │   ├── nav.rs                     # Tab bar / navigation widget
│   │   ├── header.rs                  # Title bar with Ferroscope branding
│   │   ├── footer.rs                  # Keybind hints bar
│   │   └── widgets/
│   │       ├── gauge_bar.rs           # Animated percentage bar
│   │       ├── sparkline_ext.rs       # Extended sparkline with labels
│   │       ├── flame_graph.rs         # ASCII flame graph widget
│   │       ├── thread_lane.rs         # Thread lifecycle lane chart
│   │       ├── ownership_viz.rs       # Ownership/borrow diagram renderer
│   │       ├── scope_tree.rs          # Lifetime scope bracket renderer
│   │       └── code_panel.rs          # Syntax-highlighted code snippet panel
│   │
│   ├── demos/
│   │   ├── mod.rs                     # Demo trait + registry + router
│   │   │
│   │   ├── 01_ownership/
│   │   │   ├── mod.rs                 # Ownership & Borrowing visualizer
│   │   │   ├── state.rs               # Animated ownership transfer state machine
│   │   │   └── render.rs             # Arrow/box diagram renderer
│   │   │
│   │   ├── 02_memory/
│   │   │   ├── mod.rs                 # Stack vs Heap allocation visualizer
│   │   │   ├── allocator.rs           # Custom global allocator w/ tracking
│   │   │   ├── raii.rs                # RAII Drop demonstration
│   │   │   └── render.rs
│   │   │
│   │   ├── 03_zero_cost/
│   │   │   ├── mod.rs                 # Zero-cost abstractions benchmark
│   │   │   ├── iterators.rs           # Iterator chains vs manual loops
│   │   │   ├── monomorphize.rs        # Generic monomorphization demo
│   │   │   └── render.rs
│   │   │
│   │   ├── 04_concurrency/
│   │   │   ├── mod.rs                 # Fearless concurrency showcase
│   │   │   ├── threads.rs             # std::thread spawning + join
│   │   │   ├── channels.rs            # mpsc channel message passing
│   │   │   ├── mutex_rwlock.rs        # Mutex vs RwLock contention demo
│   │   │   ├── rayon_parallel.rs      # Rayon work-stealing parallelism
│   │   │   └── render.rs
│   │   │
│   │   ├── 05_async/
│   │   │   ├── mod.rs                 # Async/await runtime visualizer
│   │   │   ├── futures.rs             # Future polling state machine
│   │   │   ├── tasks.rs               # Tokio task spawning + scheduling
│   │   │   ├── select.rs              # tokio::select! race demo
│   │   │   └── render.rs
│   │   │
│   │   ├── 06_performance/
│   │   │   ├── mod.rs                 # Live performance benchmarks
│   │   │   ├── sort_race.rs           # Sorting algorithm race (real timings)
│   │   │   ├── arithmetic.rs          # Integer/float throughput
│   │   │   ├── alloc_throughput.rs    # Allocation/deallocation speed
│   │   │   ├── simd_demo.rs           # SIMD intrinsics (std::simd / target_feature)
│   │   │   └── render.rs
│   │   │
│   │   ├── 07_type_system/
│   │   │   ├── mod.rs                 # Type system & trait showcase
│   │   │   ├── traits.rs              # Trait implementation + dispatch
│   │   │   ├── generics.rs            # Generic bounds + where clauses
│   │   │   ├── enums.rs               # Sum types, ADTs, exhaustive matching
│   │   │   ├── newtype.rs             # Newtype pattern for type safety
│   │   │   └── render.rs
│   │   │
│   │   ├── 08_error_handling/
│   │   │   ├── mod.rs                 # Error handling philosophy demo
│   │   │   ├── result_chain.rs        # Result<T,E> propagation with ?
│   │   │   ├── option_chain.rs        # Option<T> vs null-pointer crashes
│   │   │   ├── custom_errors.rs       # thiserror custom error hierarchy
│   │   │   └── render.rs
│   │   │
│   │   ├── 09_lifetimes/
│   │   │   ├── mod.rs                 # Lifetime visualizer
│   │   │   ├── scopes.rs              # Scope entry/exit animations
│   │   │   ├── dangling.rs            # Prevented dangling pointer demo
│   │   │   ├── static_lifetime.rs     # 'static and string literals
│   │   │   └── render.rs
│   │   │
│   │   ├── 10_unsafe/
│   │   │   ├── mod.rs                 # Unsafe Rust — controlled power
│   │   │   ├── raw_pointers.rs        # Raw pointer arithmetic
│   │   │   ├── ffi_demo.rs            # FFI boundary illustration
│   │   │   ├── unsafe_bounds.rs       # What unsafe enables vs what it doesn't break
│   │   │   └── render.rs
│   │   │
│   │   ├── 11_wasm/
│   │   │   ├── mod.rs                 # WebAssembly capability screen
│   │   │   ├── targets.rs             # Compile target information
│   │   │   └── render.rs
│   │   │
│   │   └── 12_system_metrics/
│   │       ├── mod.rs                 # Real-time system metrics dashboard
│   │       ├── cpu.rs                 # Per-core CPU usage via sysinfo
│   │       ├── memory.rs              # RAM / Ferroscope heap footprint
│   │       ├── process.rs             # This process's own resource usage
│   │       └── render.rs
│   │
│   └── metrics/
│       ├── mod.rs                     # Metrics aggregation
│       ├── sampler.rs                 # Background sampling thread
│       └── history.rs                 # Ring-buffer time-series storage
│
├── benches/
│   ├── sort_bench.rs                  # criterion sort benchmarks
│   ├── alloc_bench.rs                 # Allocation throughput
│   └── iter_bench.rs                  # Iterator vs loop
│
├── Cargo.toml
├── Cargo.lock
├── README.md
├── PLAN.md                            # This file
└── .gitignore
```

---

## Demo Screens — Detailed Spec

### Screen 1: Ownership & Borrowing
**Concept:** Rust's most unique feature — compile-time memory safety without GC.

**Visuals:**
- Animated ASCII boxes representing variables/values on stack and heap
- Arrows showing ownership transfer (move semantics)
- Colored borders: green = owned, yellow = borrowed immutably, red = borrowed mutably
- Live counter: "Borrow checker violations prevented: X"
- Step-by-step animation: `let a = String::from("hi")` → `let b = a` → `a` grays out (moved)
- Side panel: code snippet synced to the animation

**Key concepts shown:** move semantics, copy types, immutable borrow (&T), mutable borrow (&mut T), borrow checker enforcement

---

### Screen 2: Memory Management (Stack vs Heap)
**Concept:** Deterministic memory — no garbage collector, no runtime pauses.

**Visuals:**
- Split view: STACK (grows downward, fast) | HEAP (allocated regions)
- Live animation of `push_frame` / `pop_frame` as functions enter/exit scope
- Heap blocks appear/disappear with Box<T>, Vec<T>, String allocations
- Custom global allocator reports total bytes allocated/freed in real-time
- RAII drop animation: value crosses scope boundary → Drop trait fires → heap freed immediately
- Comparison panel: "GC language: heap X bytes unreleased | Rust: 0 bytes leaked"

**Key concepts shown:** stack vs heap, RAII, Drop trait, Box<T>, Vec<T>, zero memory leaks, no GC pauses

---

### Screen 3: Zero-Cost Abstractions
**Concept:** High-level code compiles to the same machine code as hand-written loops.

**Visuals:**
- Side-by-side live benchmark: `for i in 0..N { sum += i }` vs `(0..N).sum()`
- Frame-rate graph showing identical throughput (ns/iter)
- Scrolling assembly output panel (generated via `objdump` or inline asm inspection) showing the same instructions
- Trait object dispatch overhead visualizer: `dyn Trait` vs `impl Trait` (static dispatch)
- Monomorphization counter: "X specialized versions generated, 0 runtime overhead"

**Key concepts shown:** iterator adapters, map/filter/fold, `impl Trait`, monomorphization, static dispatch

---

### Screen 4: Fearless Concurrency
**Concept:** Data races are compile-time errors. Share data safely across threads.

**Visuals:**
- Thread lane chart: N lanes, each lane shows a thread's lifecycle (spawn → work → join)
- Message passing: animated packets traveling through `mpsc` channels between threads
- Mutex contention meter: shows threads waiting on lock (and how Rust prevents data races)
- `Arc<Mutex<T>>` reference count display live
- Rayon parallel sort: before/after timing with N-thread visualization
- Data race "attempt" panel: shows the compile error that prevents it (static analysis win)

**Key concepts shown:** `std::thread`, `mpsc`, `Arc`, `Mutex`, `RwLock`, `Send + Sync` marker traits, Rayon, fearless concurrency

---

### Screen 5: Async / Await Runtime
**Concept:** Async I/O without threads — lightweight tasks, cooperative scheduling, zero-cost futures.

**Visuals:**
- Task queue visualization: pending futures as cards in a queue
- Executor poll loop animation: reactor wakes tasks when ready
- `tokio::select!` race: two async branches racing, loser shown as canceled
- Async vs sync throughput comparison: 10,000 concurrent simulated I/O tasks
- Waker diagram: poll → pending → wake → poll again state machine
- Task memory footprint: "10,000 async tasks = X KB" vs "10,000 OS threads = X GB"

**Key concepts shown:** `async fn`, `.await`, `Future` trait, executor, waker, `tokio`, `select!`, cooperative vs preemptive

---

### Screen 6: Performance Benchmarks
**Concept:** Rust achieves C/C++ level performance with safety guarantees.

**Visuals:**
- Live racing bar chart: sorting algorithms competing in real-time (quicksort, mergesort, radix)
- ns/op counters updating in real time via `Instant::now()`
- Integer throughput: billions of ops/sec live counter
- Memory bandwidth graph: streaming read/write throughput (MB/s)
- SIMD demonstration: scalar vs SIMD vector addition speedup multiplier
- Allocator stress test: alloc + dealloc cycles/sec
- Flame-graph style rendering of where time is spent

**Key concepts shown:** zero-overhead, `Instant`, criterion benchmarking, SIMD, tight loops, cache friendliness

---

### Screen 7: Type System & Traits
**Concept:** Rust's type system is expressive, safe, and resolved entirely at compile time.

**Visuals:**
- Trait hierarchy tree: show a trait with multiple implementors, rendered as a tree
- Generic function call visualization: monomorphization spawns 3 colored copies at compile time
- Enum / ADT pattern match: exhaustive match checker animation (compiler enforces all arms)
- Newtype pattern: `Meters(f64)` vs `Feet(f64)` — prevented unit mix-up shown as blocked operation
- Where clause composer: interactive display of trait bounds combining

**Key concepts shown:** traits, generics, associated types, where clauses, enums as sum types, pattern matching, newtypes, algebraic data types

---

### Screen 8: Error Handling
**Concept:** Errors as values — no exceptions, no null, no surprises.

**Visuals:**
- `Result<T, E>` propagation chain: function call stack with `?` operator bouncing errors up
- `Option<T>` combinator chain: `.map()`, `.and_then()`, `.unwrap_or()` pipeline visualization
- Custom error type hierarchy rendered as a tree (using `thiserror`)
- "What happens in other languages" panel: null pointer exception, unhandled exception → crash
- "What happens in Rust" panel: compiler forces you to handle both Ok and Err
- Panic vs recoverable error distinction

**Key concepts shown:** `Result`, `Option`, `?` operator, `map`/`and_then`, `thiserror`, `anyhow`, panics, recoverable vs unrecoverable errors

---

### Screen 9: Lifetimes
**Concept:** The borrow checker uses lifetime annotations to ensure references never outlive their data.

**Visuals:**
- Nested scope bracket diagram: colored brackets show where each value is alive
- Reference arrow: shows a borrow pointing into a scope, turning red when it would outlive its target
- Animation: dangling reference attempt → reference "breaks" as value drops → compiler error message displayed
- `'static` lifetime: string literal shown as permanent, living for entire program
- Lifetime elision: before/after showing how compiler infers lifetimes in simple cases

**Key concepts shown:** `'a` annotations, borrow lifetimes, dangling pointer prevention, `'static`, lifetime elision rules, function signature lifetimes

---

### Screen 10: Unsafe Rust
**Concept:** Rust gives you an escape hatch — but unsafe code is explicit, isolated, and auditable.

**Visuals:**
- Safe vs `unsafe` boundary visualization: glowing red border around `unsafe {}` blocks
- Raw pointer arithmetic demo: pointer walking through an array with bounds shown
- FFI call illustration: Rust calling a simulated C function with explicit `unsafe` annotation
- "Unsafe does NOT disable the borrow checker" — show what unsafe still prevents
- Size of unsafe surface: counter of unsafe lines vs total codebase lines
- What unsafe enables: inline assembly, raw pointers, calling C, manual memory management

**Key concepts shown:** `unsafe` blocks, raw pointers `*const T` / `*mut T`, FFI, `extern "C"`, what remains safe vs what is unlocked

---

### Screen 11: WebAssembly
**Concept:** Rust compiles to WASM natively — run Rust in the browser, at near-native speed.

**Visuals:**
- WASM binary format visualization: module sections shown as colored blocks (type, function, memory, export sections)
- Binary size comparison: Rust WASM binary vs equivalent JS bundle
- Target triple display: `wasm32-unknown-unknown`, `wasm32-wasi`
- `wasm-bindgen` bridge illustration: Rust ↔ JS boundary with type-safe glue
- "You are looking at Rust in your terminal. This same code can run in Chrome." message
- Live compilation target info pulled from `std::env::consts`

**Key concepts shown:** `wasm32` target, `wasm-bindgen`, binary size, WASI, portability

---

### Screen 12: Real-Time System Metrics
**Concept:** See how lean Rust is — minimal memory footprint, no GC pauses, predictable performance.

**Visuals:**
- Per-CPU-core sparkline grid (live, via `sysinfo`)
- Total RAM vs this process's RSS / heap / stack
- GC pause simulation: "Rust process pauses: 0 ms" vs simulated GC language pause spikes
- Ferroscope's own allocator stats: total allocations, bytes in use, peak usage
- Load average and thread count
- Uptime counter with zero-GC-pause proof display

**Key concepts shown:** zero GC pauses, deterministic memory, predictable latency, lean process footprint

---

## Navigation & UX

```
┌─────────────────────────────────────────────────────────────────────┐
│  🦀 FERROSCOPE  │  Rust Capabilities Explorer  │  v0.1.0           │
├──────────────────────────────────────────────────────────────────────┤
│ [1] Ownership  [2] Memory  [3] Zero-Cost  [4] Concurrency  [5] Async│
│ [6] Perf  [7] Types  [8] Errors  [9] Lifetimes  [0] Unsafe  ...    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│                    < DEMO CONTENT HERE >                             │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│  ← → Navigate │ Space: Pause/Play │ R: Reset │ Q: Quit │ ?: Help   │
└─────────────────────────────────────────────────────────────────────┘
```

**Keyboard bindings:**
- `1`–`9`, `0` — jump directly to a demo screen
- `←` / `→` or `h` / `l` — navigate between screens
- `Space` — pause/resume current animation
- `R` — reset current demo to initial state
- `+` / `-` — increase/decrease animation speed
- `F` — toggle full-screen mode for current demo
- `E` — toggle explanation panel (concept description)
- `S` — screenshot: dump current screen to `ferroscope_screenshot.txt`
- `Q` / `Ctrl+C` — quit

---

## Implementation Phases

### Phase 1 — Foundation
**Goal:** Working TUI skeleton with navigation, theme, and the Demo trait.

- [ ] Create `src/main.rs` with Tokio async entry point
- [ ] `App` struct with selected demo, tick counter, running state
- [ ] Event loop: crossterm polling + tick timer (60 fps target)
- [ ] Tab navigation widget (numbers + arrow keys)
- [ ] Header, footer widgets
- [ ] `Demo` trait: `fn tick(&mut self, dt: Duration)`, `fn render(&self, frame: &mut Frame, area: Rect)`, `fn description() -> &'static str`
- [ ] Demo registry: `Vec<Box<dyn Demo>>`
- [ ] Color theme system (Rust-orange palette)
- [ ] Placeholder screens for all 12 demos

**Deliverable:** Navigable TUI with 12 tabs, all showing placeholder content.

---

### Phase 2 — Memory & Ownership Demos (Screens 1, 2, 9)
**Goal:** Animate Rust's most distinctive features.

- [ ] Custom global allocator wrapper (track allocs/frees in real-time)
- [ ] Ownership state machine (step animator)
- [ ] Stack/heap diagram widget with live allocation events
- [ ] RAII drop animation
- [ ] Lifetime scope bracket renderer
- [ ] Dangling reference "attempt and prevent" animation

---

### Phase 3 — Concurrency & Async Demos (Screens 4, 5)
**Goal:** Show fearless concurrency and async task scheduling live.

- [ ] Background thread pool spawner feeding demo state
- [ ] Thread lane chart widget
- [ ] mpsc channel visualizer (producer/consumer animation)
- [ ] Arc<Mutex<T>> reference count display
- [ ] Rayon parallel sort live comparison
- [ ] Tokio task queue visualization
- [ ] Future poll state machine animation
- [ ] tokio::select! race demo

---

### Phase 4 — Performance Demos (Screens 3, 6, 12)
**Goal:** Show real numbers, running live.

- [ ] Sorting algorithm live race
- [ ] Iterator vs loop zero-cost benchmark (inline timing)
- [ ] SIMD vs scalar demo
- [ ] sysinfo integration for System Metrics screen
- [ ] Background metrics sampler with ring-buffer history
- [ ] Per-core CPU sparklines
- [ ] Process RSS / heap display

---

### Phase 5 — Type System, Error Handling, Unsafe, WASM (Screens 7, 8, 10, 11)
**Goal:** Cover the remaining Rust concepts.

- [ ] Trait tree renderer
- [ ] Pattern match exhaustiveness animation
- [ ] Result/Option propagation chain visualizer
- [ ] Custom error hierarchy display
- [ ] Unsafe boundary visualizer
- [ ] WASM target info screen
- [ ] Binary section diagram

---

### Phase 6 — Polish & Release
**Goal:** Production-quality TUI, README overhaul, GitHub Actions CI.

- [ ] Full keyboard help overlay (`?`)
- [ ] Screenshot export (`S` key → `.txt` dump)
- [ ] Speed control (`+`/`-`) across all animated demos
- [ ] Explanation panel toggle (`E`) with concept descriptions
- [ ] README overhaul: demo GIFs (record with `vhs` or `asciinema`)
- [ ] GitHub Actions CI: `cargo check`, `cargo clippy`, `cargo test`, `cargo build --release`
- [ ] `cargo build --target wasm32-unknown-unknown` CI step to verify WASM builds
- [ ] Release binary for macOS, Linux, Windows via `cargo-dist` or GitHub Actions matrix
- [ ] Publish to crates.io

---

## Dependency Justification (Why Each Crate)

| Crate | Purpose | Why not X? |
|-------|---------|------------|
| `ratatui` | TUI framework | Active fork of tui-rs, maintained, widely used |
| `crossterm` | Terminal backend | Cross-platform (Win/Mac/Linux), no ncurses dep |
| `tokio` | Async runtime | De-facto standard; demonstrates the async ecosystem |
| `rayon` | Parallelism | Best-in-class work-stealing; Rust-idiomatic |
| `sysinfo` | System metrics | Pure Rust, cross-platform, actively maintained |
| `parking_lot` | Fast sync primitives | Faster than std::sync, good showcase of ecosystem |
| `crossbeam` | Advanced concurrency | Lock-free data structures, scoped threads |
| `rand` | Randomness | Needed for benchmark data generation |
| `anyhow` / `thiserror` | Error types | Show both "application" and "library" error patterns |
| `instant` | WASM-safe timing | `std::time::Instant` doesn't compile to WASM |
| `criterion` | Benchmarks | Standard Rust benchmarking; bench/ targets |

---

## WASM Deployment Strategy

Ferroscope's core logic is written to compile to both native and `wasm32-unknown-unknown`.

**Native (primary):** Full TUI via `ratatui` + `crossterm`.

**WASM (secondary):** A subset of the demo logic (benchmarks, visualizations, metrics that don't require `sysinfo`) compiles to WASM and can be embedded in a static HTML page using `xterm.js` as the terminal emulator backend, OR rendered as canvas animations directly via `web-sys`.

```
cargo build --target wasm32-unknown-unknown --release
wasm-pack build --target web
```

The web version can be deployed to GitHub Pages directly from the repo, making Ferroscope instantly shareable via a URL — no install required.

---

## Quality Standards

- **`cargo clippy -- -D warnings`** must pass clean
- **`cargo fmt`** enforced via CI
- **`cargo test`** for all demo logic units
- **`cargo audit`** for dependency vulnerability scanning
- **No `unwrap()` in library code** — all errors handled with `?` or explicit match
- **Unsafe code** limited to Screen 10's demo block, with `// SAFETY:` comments on every `unsafe` block
- Doc comments (`///`) on all public items

---

## Sharing Strategy

| Format | How |
|--------|-----|
| Terminal binary | `cargo install ferroscope` (crates.io) |
| Browser / shareable URL | GitHub Pages WASM build (auto-deployed via CI) |
| Demo GIFs | Recorded with `vhs` or `asciinema`, embedded in README |
| Blog post substrate | Each screen's explanation panel = ready-made article sections |

---

## Success Criteria

A person who has never written Rust should be able to run `cargo install ferroscope`, navigate the 12 screens in 10 minutes, and come away understanding:

1. Why Rust has no garbage collector and why that matters
2. What "ownership" and "borrowing" mean concretely
3. Why "fearless concurrency" is not a marketing slogan
4. How Rust achieves C-level performance with high-level abstractions
5. That Rust can run in the browser via WebAssembly
6. Why Rust's error handling is superior to exceptions
7. What "zero-cost abstractions" means with numbers to prove it

Output assets/demo.gif

Set Shell    "bash"
Set FontSize 13
Set Width    1440
Set Height   810
Set Framerate 15
Set Theme    "Dracula"
Set Padding  16

# ── cd into the project (hidden so the shell prompt isn't shown) ─────────────
Hide
Type "cd /Users/wscholl/ferroscope"
Enter
Sleep 0.5s
Show

# ── Launch the app ────────────────────────────────────────────────────────────
Type "./target/release/ferroscope --fps 15"
Enter
Sleep 2.5s

# 1 — Ownership & Borrowing (already on screen at launch)
Sleep 2s

# 2 — Memory Management
Type "2"
Sleep 2s

# 3 — Zero-Cost Abstractions
Type "3"
Sleep 2s

# 4 — Fearless Concurrency
Type "4"
Sleep 2.5s

# 5 — Async/Await
Type "5"
Sleep 2s

# 6 — Performance Benchmarks (live benchmarks — longer dwell)
Type "6"
Sleep 3s

# 7 — Type System
Type "7"
Sleep 2s

# 8 — Error Handling
Type "8"
Sleep 2s

# 9 — Lifetimes
Type "9"
Sleep 2s

# 10 — Unsafe Rust
Type "0"
Sleep 2s

# 11 — WebAssembly (4-step step carousel)
Type "a"
Sleep 2.5s

# 12 — System Metrics (live CPU/mem — longer dwell)
Type "b"
Sleep 3s

# Toggle explanation panel to show the feature
Type "e"
Sleep 1.5s
Type "e"

# 13 — Compile-Time Guarantees
Type "c"
Sleep 2s

# 14 — Cargo Ecosystem
Type "d"
Sleep 2s

# 15 — Embedded / no_std
Type "f"
Sleep 2s

# Speed up to show the speed control feature
Type "+"
Type "+"
Sleep 1.5s

# Loop back to Demo 1 for a clean ending frame
Type "1"
Sleep 2s

# Quit
Type "q"

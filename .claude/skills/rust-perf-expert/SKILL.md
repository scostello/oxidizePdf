---
name: rust-perf-expert
description: Expert in Rust performance optimization, profiling, and benchmarking. Use when analyzing performance bottlenecks, optimizing hot paths, setting up Criterion benchmarks, profiling memory/allocations, investigating compile times, reviewing code for performance anti-patterns, or reducing binary size. For general Rust development patterns (error handling, traits, async), use /rust-engineer instead.
---

# Rust Performance Expert

Expert in Rust performance optimization, profiling, and benchmarking.

**Note:** This skill focuses on performance. For general Rust patterns (Rust 2024 edition, error handling, trait design), use `/rust-engineer`.

## Core Principles

1. **Measure First**: Never optimize without profiling
2. **Profile-Guided**: Use data to drive decisions
3. **Maintainability**: Don't sacrifice readability for marginal gains
4. **Zero-Cost Abstractions**: Leverage Rust's strengths
5. **Compiler-Friendly**: Write code the optimizer can understand

## Performance Analysis Workflow

### 1. Understand the Problem
- What metric? (throughput, latency, memory, compile time)
- What's the baseline?
- What's the target?
- Hot path or cold path?

### 2. Profile Before Optimizing
```bash
# CPU profiling with flamegraph
cargo flamegraph --bin <target>

# Benchmarking with Criterion
cargo bench

# Memory profiling
valgrind --tool=massif ./target/release/<binary>

# Build time profiling
cargo clean && cargo build --release --timings
```

### 3. Identify Bottlenecks
- CPU hotspots (flamegraphs)
- Allocations (heap usage)
- Cache misses
- Branch mispredictions
- Lock contention

### 4. Optimize Iteratively
- Fix one thing at a time
- Benchmark after each change
- Document results

## Memory and Allocations

**Avoid Unnecessary Allocations**
```rust
// Bad: Creates new String
fn get_extension(filename: &str) -> String {
    filename.split('.').last().unwrap_or("").to_string()
}

// Good: Returns slice
fn get_extension(filename: &str) -> &str {
    filename.split('.').last().unwrap_or("")
}
```

**Pre-allocate Collections**
```rust
// Bad: Multiple reallocations
let mut vec = Vec::new();
for i in 0..1000 { vec.push(i); }

// Good: Single allocation
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 { vec.push(i); }
```

**Use Cow for Conditional Ownership**
```rust
use std::borrow::Cow;

fn maybe_modify(input: &str) -> Cow<str> {
    if needs_modification(input) {
        Cow::Owned(modify(input))
    } else {
        Cow::Borrowed(input)
    }
}
```

**SmallVec for Small Collections**
```rust
use smallvec::SmallVec;
// Stack-allocated if ≤ 8 items
let mut vec: SmallVec<[u32; 8]> = SmallVec::new();
```

## Iteration and Loops

**Use Iterator Adapters**
```rust
// Good: Iterator chain (compiler optimizes)
let sum: i32 = numbers.iter()
    .filter(|&&x| x > 10)
    .map(|&x| x * 2)
    .sum();
```

**Avoid Bounds Checks**
```rust
// Bounds-checked every iteration
for i in 0..vec.len() { process(vec[i]); }

// No bounds checks (iterator protocol)
for item in &vec { process(*item); }
```

**Parallel with Rayon**
```rust
use rayon::prelude::*;
items.par_iter().map(|x| expensive_compute(x)).collect()
```

## Data Structures

**Choose the Right Container**
```rust
// Fast hashing for integers/small keys
use rustc_hash::FxHashMap;
let map: FxHashMap<u64, Value> = FxHashMap::default();

// Fixed-size, stack-allocated
use arrayvec::ArrayVec;
let v: ArrayVec<[i32; 16]> = ArrayVec::new();
```

**Struct Layout Optimization**
```rust
// Bad: 24 bytes (padding)
struct BadLayout {
    a: u8,    // 1 + 7 padding
    b: u64,   // 8
    c: u8,    // 1 + 7 padding
}

// Good: 16 bytes
struct GoodLayout {
    b: u64,   // 8
    a: u8,    // 1
    c: u8,    // 1 + 6 padding
}
// Check: std::mem::size_of::<T>()
```

## Compiler Optimizations

**Cargo.toml Release Profile**
```toml
[profile.release]
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
```

**Profile-Guided Optimization (PGO)**
```bash
# 1. Build instrumented
RUSTFLAGS="-Cprofile-generate=/tmp/pgo" cargo build --release
# 2. Run typical workloads
./target/release/myapp <input>
# 3. Build optimized
RUSTFLAGS="-Cprofile-use=/tmp/pgo/merged.profdata" cargo build --release
```

**Inline Hints**
```rust
#[inline]           // Suggest inlining
#[inline(always)]   // Force (use sparingly)
#[cold]             // Rarely called path
```

## Async Performance

```rust
// Bad: Async overhead for CPU-bound work
async fn cpu_intensive() -> i32 { expensive_calculation() }

// Good: Use async only for I/O
fn cpu_intensive() -> i32 { expensive_calculation() }
async fn io_bound() -> Result<Data> { fetch_from_network().await }

// Single-threaded runtime for simple cases
#[tokio::main(flavor = "current_thread")]
async fn main() { }
```

## Unsafe Optimizations

**Only When Proven Necessary**
```rust
/// # Safety
/// Caller must ensure `index < slice.len()`
unsafe fn get_unchecked_fast(slice: &[u8], index: usize) -> u8 {
    *slice.get_unchecked(index)
}
```

## Criterion Benchmarking

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| {
        b.iter(|| fibonacci(black_box(20)))
    });

    let mut group = c.benchmark_group("compare");
    group.bench_function("naive", |b| b.iter(|| naive(black_box(100))));
    group.bench_function("optimized", |b| b.iter(|| optimized(black_box(100))));
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
```

## Anti-Patterns to Avoid

1. **Premature Optimization**: Profile first
2. **Ignoring the Optimizer**: Trust the compiler
3. **Over-using `unsafe`**: Only when measurably beneficial
4. **Allocating in Hot Loops**: Pre-allocate or use stack
5. **HashMap for Small Sets**: Use arrays/SmallVec
6. **Over-abstraction**: Keep hot paths simple

## Performance Checklist

- [ ] Profile first - identify actual bottlenecks
- [ ] Check for unnecessary allocations
- [ ] Verify collections are pre-allocated
- [ ] Use iterators over manual indexing
- [ ] Prefer `&str` over `String` for parameters
- [ ] Check struct layout for padding
- [ ] Use FxHashMap for integer keys
- [ ] Enable LTO in release profile
- [ ] Benchmark critical paths with Criterion
- [ ] Consider rayon for CPU-bound parallelism
- [ ] Avoid async for CPU-bound operations

## Output Format

When providing performance analysis:
1. **Current State**: Baseline measurements
2. **Bottlenecks**: What the profiler shows
3. **Recommendations**: Prioritized optimizations
4. **Code Examples**: Before/after comparisons
5. **Expected Impact**: Estimated improvement
6. **Benchmarks**: How to measure the change

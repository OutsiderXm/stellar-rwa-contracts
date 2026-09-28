# Fuzzing Guide

Issue #378: Fuzzing for arithmetic-heavy paths

This directory contains fuzz targets for the RWA contracts, focusing on arithmetic operations that handle i128 overflow/underflow and boundary conditions.

## Building and Running Fuzz Tests

### Prerequisites

Install `cargo-fuzz`:
```bash
cargo install cargo-fuzz
```

### Running Fuzz Tests

Run the dividend arithmetic fuzzer:
```bash
cd fuzz
cargo fuzz run fuzz_dividend_arithmetic
```

By default, this will run indefinitely. To limit execution time:
```bash
cargo fuzz run fuzz_dividend_arithmetic -- -max_len=1000 -max_total_time=300
```

### Interpreting Results

The fuzzer will:
1. Generate random distributions with varying holder balances
2. Execute claim operations
3. Assert that total claims never exceed the pool
4. Report any crashes or assertion failures with minimal reproducer

### Filing Findings

If the fuzzer discovers a crash or violation:
1. A crash file will be saved (e.g., `artifacts/fuzz_dividend_arithmetic/crash-*`)
2. Create a new issue with:
   - The crash/violation description
   - The minimal reproducer (the crash file)
   - Steps to reproduce
   - Expected vs. actual behavior

### Current Coverage

- **dividend arithmetic**: claim total bounds, overflow detection, dust behavior

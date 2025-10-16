# Copilot Instructions for modern-alchemy

This project is a Rust reimplementation of Walter Fontana's Alchemy, simulating lambda calculus-based chemical systems. It integrates Rust, Python (via PyO3), and external crates for simulation and analysis.

## Architecture Overview
- **Core Simulation**: Main logic in `src/` (`main.rs`, `lib.rs`, `analysis.rs`). Lambda calculus operations are in `src/lambda/`.
- **Configuration**: Simulation parameters are defined in `src/config.rs` and `default/config.json`. The `Config` struct is the single source of truth.
- **Python Bindings**: Core library functionality is exposed to Python via PyO3 in `src/lib.rs`, built with [Maturin](https://github.com/PyO3/maturin).

## Developer Workflows
- **Build**: `cargo build`
- **Run**: `cargo run -- --config default/config.json`
- **Pipe Input**: `python /path/to/fontana_generator.py | cargo run -- --config default/config.json`
- **Testing**: No standard Rust test suite. Validate via experiment scripts or manual runs. For Python bindings, use `test_alchemy.py`.
- **Dashboard Integration**: Output is consumed by [Alchemy-Dashboard](https://github.com/mathis-group/Alchemy-Dashboard) (separate project; requires Python wrappers).

## Project-Specific Patterns
- **Config-Driven**: All runs depend on `default/config.json` and the `Config` struct.
- **Data Flow**: Input is typically piped via `stdin`; output is written to files or `stdout` for dashboard consumption.
- **External Dependencies**: Uses the `lambda-btree` crate and Python scripts for input generation.

## Python Bindings
- **Bindings Location**: Implemented in `src/lib.rs` using PyO3.
- **Exposed Types**: `PyReactor`, `PyStandardization`, `PySoup`, `PyBTreeGen`, `PyFontanaGen`, plus utility functions (`encode_hex_py`, `decode_hex_py`).
- **Experiment Functions**: Some (e.g., `run_entropy_series`) are intended for Python but may be commented out; uncomment and wrap as needed.
- **Testing**: Use `test_alchemy.py` to validate Python API.
- **Build/Install**: `maturin develop` (in project root), then `import alchemy` in Python.
- **Error Handling**: Custom error types (e.g., `PyReactionError`) are mapped for Python compatibility.
- **Adding Bindings**: Use `#[pyclass]`, `#[pymethods]`, `#[pyfunction]` in `src/lib.rs` and update `test_alchemy.py`.

## Key Files & Directories
- `src/` — Rust source code
- `src/experiments/` — Experiment modules
- `src/lambda/` — Lambda calculus logic
- `default/config.json` — Main configuration
- `scripts/` — Automation scripts
- `test_alchemy.py` — Python API test script

---
If any section is unclear or incomplete, please provide feedback or specify which workflows or patterns need more detail.
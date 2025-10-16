# Copilot Instructions for modern-alchemy

This project is a Rust reimplementation of Walter Fontana's Alchemy, focused on simulating lambda calculus-based chemical systems. It integrates Rust, Python, and external crates for simulation and analysis.

## Architecture Overview
- **Core Simulation**: Main logic in `src/`, especially `main.rs`, `lib.rs`, and `analysis.rs`. Lambda calculus operations are in `src/lambda/`.
- **Experiments**: Modular experiment logic in `src/experiments/` (e.g., `discovery.rs`, `kinetics.rs`). Each file is a distinct experiment type. Experiments are managed manually by individual contributors and not through a unified interface, we don't need to ensure backwards compatibility with these files.
- **Config**: Configuration of simulations is handled via `src/config.rs` and `default/config.json`. The `Config` object is the source of truth for simulation parameters.
- **Python Integration**: Python integration is under development. The goal is to wrap the core library functionality in Python using PyO3 and [Maturin](https://github.com/PyO3/maturin).


## Developer Workflows
- **Build**: `cargo build`
- **Run**: `cargo run -- {args}`
- **Test**: Most testing is done by running experiments with specific arguments. No standard Rust test suite; use experiment scripts and manual runs.
- **Documentation**: `cargo doc --open` for Rust API docs. The `Config` object is documented in code.
- **Interactive Dashboard**: For visualization, see [Alchemy-Dashboard](https://github.com/mathis-group/Alchemy-Dashboard). This is a separate project that consumes output from this simulation, it needs support from the Python wrappers to be fully functional.

## Project-Specific Patterns
- **Experiment Modularity**: Add new experiments by creating a new file in `src/experiments/` and updating `mod.rs`.
- **Config-Driven**: All runs depend on the configuration in `default/config.json` and the `Config` Rust struct.
- **Data Flow**: Input is typically piped via `stdin` (e.g., from Python generators), output is written to files or stdout for dashboard consumption.
- **External Dependencies**: Relies on the `lambda-btree` crate and Python scripts for input generation.

## Examples
- Run a simulation: `cargo run -- --config default/config.json`
- Pipe input: `python /path/to/fontana_generator.py | cargo run -- --config default/config.json`
- Batch experiments: `bash scripts/discovery.sh`

## Key Files & Directories
- `src/` — Rust source code
- `src/experiments/` — Experiment modules
- `src/lambda/` — Lambda calculus logic
- `default/config.json` — Main configuration
- `scripts/` — Automation scripts

---
For unclear or incomplete sections, please provide feedback or specify which workflows or patterns need more detail.
## Python Bindings Development

- **Bindings Location**: Python bindings are implemented in `src/lib.rs` using PyO3. The goal is to expose core simulation types and utilities to Python.
- **Exposed Classes/Functions**: The following Rust types are wrapped for Python:
	- `PyReactor`
	- `PyStandardization`
	- `PySoup`
	- `PyBTreeGen`
	- `PyFontanaGen`
	- Utility functions: `encode_hex_py`, `decode_hex_py`
- **Experiment Functions**: Some experiment functions (`run_entropy_series`, `run_entropy_test`, `run_sync_entropy_test`) are intended to be exposed to Python, but are currently commented out in `src/lib.rs`. Uncomment and wrap these as needed.
- **Testing Python Bindings**: Use `test_alchemy.py` to validate Python API. This script exercises all major classes and functions, including experiment functions.
- **Build/Install for Python**: Use [Maturin](https://github.com/PyO3/maturin) to build and install the Python module. Example:
	- Build: `maturin develop` (in project root)
	- Import in Python: `import alchemy`
- **Error Handling**: Custom error types (e.g., `PyReactionError`) are mapped from Rust errors for Python compatibility.
- **Adding New Bindings**: To expose new Rust functions or types to Python, use `#[pyclass]`, `#[pymethods]`, and `#[pyfunction]` macros in `src/lib.rs`, then update the Python test script to cover new features.
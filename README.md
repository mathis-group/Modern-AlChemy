A reimplementation of Walter Fontana's Alchemy. Pipe lambda expressions into 
`stdin` to start a default simulation. 

Usage:

`alchemy`

Build: 

`cargo build`

Testing:

`cargo run -- {args}`

With the binary tree generators from the 
[lambda-btree](https://github.com/AgentElement/lambda-btree) crate, you can
run a simple alchemy simulation with the following command.

`python /path/to/src/fontana_generator.py | cargo run -- {args}`


Documentation:

* Full documentation: `cargo doc --open`
* Help: `cargo run -- --help`

The documentation for the configuration file is in the `Config` object.


# Interactive Dashboard

Can be found here: https://github.com/mathis-group/Alchemy-Dashboard


# Steps to run updated

# 0) Pick & activate the EXACT Python you will use (avoid shim confusion)
#    (choose one)
# — pyenv:
pyenv shell 3.11.9

# — OR virtualenv:
# python3.11 -m venv .venv && source .venv/bin/activate

# 1) Show which Python we’re about to use
python -c 'import sys,platform; print(sys.executable); print(platform.python_version(), platform.machine())'

# 2) Clean previous artifacts
cargo clean
python -m pip uninstall -y alchemy || true
rm -rf target

# 3) Build & install the package *editable* via the PEP 517 backend (maturin)
python -m pip install -e .

# 4) Sanity-check the import (this uses the same interpreter as above)
python - <<'PY'
import alchemy, sys
print("OK :", alchemy.__file__)
print("Py :", sys.executable)
PY

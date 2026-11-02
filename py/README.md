# `vexio` Python bindings

This directory contains PyO3 Python bindings for `vexio`.

## Build and install for development

From this directory, with Python and Maturin installed:

```bash
python -m pip install maturin
maturin develop
```

To build a wheel:

```bash
maturin build --release
```

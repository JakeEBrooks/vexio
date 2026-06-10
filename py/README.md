# `vexio` — Python Bindings

Python bindings for [`vexio`](https://github.com/JakeEBrooks/vexio), the friendliest VLBI Experiment (VEX) file support library.

## Installation

Build from source using [maturin](https://github.com/PyO3/maturin):

```bash
pip install maturin
maturin develop
```

Or build a wheel for installation:

```bash
maturin build --release
pip install target/wheels/vexio-*.whl
```

## Quick Start

```python
import vexio

# Parse a VEX file from disk
vex = vexio.Vex.open("experiment.vex")

# Construct programmatically
vex = vexio.Vex()
mydef = vexio.Def("myblock")
mydef.add_param(vexio.Param("param1", ["value"]))

# Serialize back to VEX text
print(vex.to_vex())

# Save to a file
vex.save("output.vex")
```

See also the [examples](./examples)

## License

MIT or Apache-2.0, same as the core `vexio` library. See the [parent project](https://github.com/JakeEBrooks/vexio) for details.

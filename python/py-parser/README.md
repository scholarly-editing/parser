# Python binding

`py-parser` exposes the parser's passage lookup functionality as a Python extension module named `py_parser`. It currently provides a `Passage` class and accepts TOML configuration text.

## Install for local development

Requirements: Rust, Python 3.8 or newer, and Maturin 1.7 or newer but below 2.0. Activate a Python virtual environment, then from this directory run:

~~~sh
python -m pip install "maturin>=1.7,<2.0"
maturin develop
~~~

## Example

The TOML deserializer requires the `prefix`, `brackets`, and `tag` collections in addition to at least one `word` definition:

~~~python
from py_parser import Passage

config = """
prefix = []
brackets = []
tag = []

[[word]]
label = "Arabic"
char_range = [0x0600, 0x06FF]
additional_chars = []
default_state = "sound"
"""

passage = Passage("باب الأسد والثور", config)
print(passage.is_valid())                 # True when parsing found no errors
print(passage.get_raw_passage())           # باب الأسد والثور
print(passage.locate_fragment("الأسد"))    # token-order index pair
~~~

`locate_fragment` returns a pair of token-order indexes, or `(-1, -1)` if the fragment is not found. `get_raw_passage` returns the recognized words joined by spaces and is empty for an invalid passage. Supply valid TOML: the current Python constructor unwraps configuration parsing errors.

The extension crate is in this directory; the shared parser and configuration examples are linked from the [repository overview](../../README.md).

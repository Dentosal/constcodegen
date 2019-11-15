# ConstantCodeGen - Share constants between languages

Features:
* Language agnostic: supports (almost) any language
* Generates header files or replaces placeholders

Planned features:
* Read from JSON in addition to TOML
* Constraints for automatically checking values for validity
* Data format exports, JSON etc.
* Proper, autogenerated documentation

Non-goals:
* Turing complete configuration language
* Sharing code - maybe small pure functions but not more

## Try it!

```bash
cargo run -- --target-dir target/ --options-file examples/options.toml -c examples/constants.toml
cat target/constants.rs
cat target/constants.py
```
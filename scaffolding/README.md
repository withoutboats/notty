This is a scaffolding terminal for testing the notty terminal library. It uses
the notty-cairo rendering library in the same repository.

__NOTE:__ This terminal is buggy, incomplete, and not intended for general use.

# Building

## Rust version

Currently notty requires rust nightly. Install from [rust-lang.org][rust_dl]
or use [multirust][multirust].

## Other requirements

This library depends on GTK, Pango, and Cairo. [GTK-rs][gtk-rs] has install
instructions for Mac OS X and Linux.

[gtk-rs]: https://github.com/gtk-rs/gtk
[multirust]: https://github.com/brson/multirust
[rust_dl]: https://www.rust-lang.org/downloads.html

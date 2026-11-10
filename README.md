# waterui-barcode

QR code and barcode generation component for WaterUI.

Barcodes are drawn as vector geometry on a `waterui_graphics::canvas::Canvas`,
so whichever renderer draws the screen around a barcode draws it. Dark modules
are filled by a color, a gradient, or any canvas content through
`Barcode::fill_scene`.

## Example

`examples/barcode` is a WaterUI playground that renders the symbologies this
crate supports. It is a member of this workspace, so it builds against the
component source in this repository:

```bash
cd examples/barcode
water run --platform macos
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

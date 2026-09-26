# waterui-barcode

QR code and barcode generation component for WaterUI.

Barcodes record vector paths and reactive paints through Cherenkov. The engine
retains their geometry, clipping and color bindings across frames.

With the `gpu` feature, `BarcodeSource::generate` renders a standalone image
through `OffscreenRenderer`. Drawing a barcode into a view needs no GPU feature.

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

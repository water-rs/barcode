# waterui-barcode

QR code and barcode generation component for WaterUI.

Barcodes are drawn as vector geometry through `waterui-graphics`' engine-neutral
`Scene2D` contract, so the same component renders on the GPU compute renderer,
the CPU sparse-strip renderer used on embedded targets, and any backend that
owns its own scene.

Rasterizing a barcode into a standalone image needs a GPU device, so the
`ImageGenerator` implementation for `BarcodeSource` sits behind the non-default
`gpu` feature. Drawing a barcode into a view does not.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

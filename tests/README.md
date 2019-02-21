## Fixtures

Here we describe all of our test fixtures along with the original reason that they were created.

Note that over time we might re-use our existing test fixtures in different ways - so these only
only describe the original motivation for creating them, not every way that they may be used right now.

#### green-1x1.psd

1x1 pixels.

A PSD file with a single green pixel.

The original `Background` layer was deleted and a layer called `First Layer` was created.

This allowed for the layer to show up in the layer and mask information section of the PSD file.

#### two-layers-red-green-1x1.psd

1x1 pixels.

A PSD file with two layers. The bottom layer is green and the top layer is red.

This was originally created to test our final flattened image data in the image data section
by ensuring what we return a red image.

# rle-3-layer-8x8.psd

8x8 pixels.

A PSD with 3 layers. Bottom layer is red, middle green, top is blue

All layers as well as the final image channel are RLE compressed.

This was originally created to allow us to test that when returning pixel data we correctly factor in RLE compression.

## Fixtures

Here we describe all of our test fixtures along with the original reason that they were created.

Note that over time we might re-use our existing test fixtures in different ways - so these only
only describe the original motivation for creating them, not every way that they may be used right now.

#### green-1x1.psd

A PSD file with a single green pixel.

The original `Background` layer was deleted and a layer called `First Layer` was created.

This allowed for the layer to show up in the layer and mask information section of the PSD file.

#### two-layers-red-green-1x1.psd

A PSD file with two layers. The bottom layer is green and the top layer is red.

This was originally created to test our final flattened image data in the image data section
by ensuring what we return a red image.

#### transparent-top-layer-2x1.psd

Three layers.

Top layer has left pixel transparent, right pixel blue.

Middle layer green, bottom layer red.

This was originally created in order to test our `flatten_layers` method.
If it worked properly we should see blue on right right but a lower layer on the left
since the left of the top most layer is transparent.

#### 3x3-opaque-center.psd

3x3 grid of pixels with all transparent except for an opaque middle blue pixel.

Originally created to test having a layer where the layer's dimensions are smaller than the PSD dimensions.

#### 9x9-rle-opaque-center.psd

9x9 grid of pixels with all transparent except for an opaque middle red pixel.

Originally created to test having an rle compressed layer where the layer's dimensions are smaller
than the PSD dimensions.

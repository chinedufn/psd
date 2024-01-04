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

3x3 grid of pixels with all transparent except for an opaque middle blue pixel and top right blue pixel.

Originally created to test having a layer where the layer's dimensions are smaller than the PSD dimensions.

#### 16x16-rle-partially-opaque.psd

16x16 grid of pixels with all transparent except for an opaque block of 9x9 rle compressed red pixels,

Originally created to test having an rle compressed layer where the layer's dimensions are smaller
than the PSD dimensions.

#### transparent-above-opaque.psd

1x1 pixel PSD with top layer transparent bottom layer blue.

Originally created to test an error where we were borrowing a RefCell twice while recursively flattening a transparent pixel.

## one-channel-1x1.psd

1x1 gray image that only has one channel.

It was getting an index out of range for slice error since we assumed that there were always 3+ channels.

Color mode is grayscale.

## two-channel-8x8.psd

8x8 image with different shades of black/white/grey

It has two channels, red and green.

Color mode is grayscale.

## negative-top-left-layer.psd

A PSD file with a single layer that has an X,Y position of (-4px, -4px) and a width and height of
9x9 even though the PSD's size has been set to 1x1

This happened while resizing a PSD from 1024x1024 down to 1x1.

## green-chinese-layer-name-1x1.psd

Support for unicode encoded layer names

https://github.com/chinedufn/psd/issues/4

## green-cyrillic-layer-name-1x1.psd

Support for unicode layer names

https://github.com/chinedufn/psd/issues/4


## out-of-bounds-layer.psd
Support for layers that are the same size as PSD but is offset negatively on x and y outside of the PSD bounds

https://github.com/chinedufn/psd/issues/45
https://github.com/chinedufn/psd/issues/43

## layer-larger.psd
Contains a layer that is larger 13x9px that than the PSD bounds 1x1.

https://github.com/chinedufn/psd/issues/45
https://github.com/chinedufn/psd/issues/43

# rectangle-pack [![Actions Status](https://github.com/chinedufn/rectangle-pack/workflows/test/badge.svg)](https://github.com/chinedufn/rectangle-pack/actions) [![docs](https://docs.rs/rectangle-pack/badge.svg)](https://docs.rs/rectangle-pack)

> Pack multiple rectangles into multiple rectangles. A flexible rectangle packer built with texture atlases in mind.

`rectangle-pack` is a library focused on laying out any number of smaller rectangles inside any number of larger rectangles.

`rectangle-pack` seeks to expose an API that gives the consumer control over how rectangles are packed - allowing them to tailor
the packing to their specific use case.

While `rectangle-pack` is designed with texture atlas related use cases in mind - the library itself has no notions of images and can be used
in any context.

## Background / Initial Motivation

I'm working on a game with some of the following texture atlas requirements (as of March 2020):

- I need to be able to guarantee that certain textures are available in the same atlas.
    - For example - if I'm rendering terrain using a blend map that maps each channel to a color / metallic-roughness / normal texture
      I want all of those textures to be available in the same atlas.
      Otherwise in the worst case I might need over a dozen texture atlases in order to render a chunk of terrain.

- I want to have control over which channels are used when I'm packing my atlases.
    - For example - I need to be able to easily pack my metallic and roughness textures into one channel each, while
      packing color and normal channels into three channels.
        - This means that my rectangle packer needs to expose configuration on the number of layers/channels available in our target bins.

- I need to be able to ensure that uncommon textures aren't taking up space in commonly used atlases
    - For example - if a set of textures is only used in one specific region of the game - they shouldn't take up space in an atlas that contains a texture
      that is used for the user interface.
        - This means that the packer needs to have some notion of groups or priority so that uncommon textures can be placed separately from common ones.
    - This allows us to minimize the number of textures in GPU memory at any time since atlases with uncommon textures can be removed after not being in use for some time.
        - Without meeting this requirement - a large texture might be sitting on the GPU wasting space indefinitely since it shares an atlas with very common textures that will never be evicted.

- I need to be able to pack individual bits within a channel. For example - if I have a texture mask that encodes whether or not a fragment is metallic I want to be able to pack that into a single bit,
  perhaps within my alpha channel.
    - This means that our layers concept needs to support multi-dimensional needs. A layer within a layer.
        - For example - In color space one might be thinking of RGBA channels / layers or be thinking about within the Alpha channel having 255 different sub-layers. Or even a smaller number of variable sized sub-layers.
          Our API needs to make this simple to represent and pack.
    - We don't necessarily need to model things that way internally or even expose a multi-layered notion in the API - we just need to enable those use cases - even if we still think of things as one dimension of layers at the API level.
        - In fact .. as I type this .. one dimensions of layers at the API level both internally and externally sounds much simpler. Let the consumer worry about whether a channel is considered one layer (i.e. alpha) or 255 layers (i.e. every bit in the alpha channel).

## See Also

- [rectpack2D](https://github.com/TeamHypersomnia/rectpack2D)
    - Inspired our initial implementation

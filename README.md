# rectangle-pack [![Actions Status](https://github.com/chinedufn/rectangle-pack/workflows/test/badge.svg)](https://github.com/chinedufn/rectangle-pack/actions) [![docs](https://docs.rs/rectangle-pack/badge.svg)](https://docs.rs/rectangle-pack)

> A minimal, zero dependency rectangle packer capable of handling simple and complex use cases alike. Supports both two and three dimensions.

`rectangle-pack` is a library focused on laying out any number of smaller rectangles (both 2d rectangles and 3d rectangular prisms) inside any number of larger rectangles.

`rectangle-pack` exposes an API that gives the consumer control over how rectangles are packed - allowing them to tailor
the packing to their specific use case.

While `rectangle-pack` was originally designed with texture atlas related use cases in mind - the library itself has no notions of images and can be used
in any rectangle packing context.

## Background / Initial Motivation

I'm working on a game with some of the following texture atlas requirements (as of March 2020):

- I need to be able to guarantee that certain textures are available in the same atlas.
    - For example - if I'm rendering terrain using a blend map that maps each channel to a color / metallic-roughness / normal texture
      I want all of those textures to be available in the same atlas.
      Otherwise in the worst case I might need over a dozen texture uniforms in order to render a single chunk of terrain.

- I want to have control over which channels are used when I'm packing my atlases.
    - For example - I need to be able to easily pack my metallic and roughness textures into one channel each, while
      packing color and normal channels into three channels.
        - This means that my rectangle packer needs to expose configuration on the number of layers/channels available in our target bins.

- I need to be able to ensure that uncommon textures aren't taking up space in commonly used atlases
    - For example - if a set of textures is only used in one specific region of the game - they shouldn't take up space in an atlas that contains a texture
      that is used for very common game elements.
        - This means that the packer needs to cater to some notion of groups or priority so that uncommon textures can be placed separately from common ones.
    - This allows us to minimize the number of textures in GPU memory at any time since atlases with uncommon texture atlases can be removed after not being in use for some time.
        - Without meeting this requirement - a large texture might be sitting on the GPU wasting space indefinitely since it shares an atlas with very common textures that will never be evicted.
    - Note that we might not end up achieving this at the API level. This could potentially be achieved by just having the consumer call the library multiple times using whichever input rectangles they determine to be of
      similar priority.
        - Or some other solution.

- I need to be able to pack individual bits within a channel. For example - if I have a texture mask that encodes whether or not a fragment is metallic I want to be able to pack that into a single bit,
  perhaps within my alpha channel.
    - This means that our layers concept needs to support multi-dimensional needs. A layer within a layer.
        - For example - In color space one might be thinking of RGBA channels / layers or be thinking about within the Alpha channel having 255 different sub-layers. Or even a smaller number of variable sized sub-layers.
          Our API needs to make this simple to represent and pack.
    - We don't necessarily need to model things that way internally or even expose a multi-layered notion in the API - we just need to enable those use cases - even if we still think of things as one dimension of layers at the API level.
        - In fact .. as I type this .. one dimensions of layers at the API level both internally and externally sounds much simpler. Let the consumer worry about whether a channel is considered one layer (i.e. alpha) or 255 layers (i.e. every bit in the alpha channel).

- I need to be able to allow one texture to be present in multiple atlases.
    - For example - say there is a grass texture that is used in every grassy region of the game. Say each of those regions has some textures that are only used in that region and thus relegated to their own
      atlas. We want to make sure our grass texture is copied into each of those textures so that one texture can support the needs of that region instead of two.

These requirements will be used as guiding pillars to design a flexible API.

The API shouldn't know about the specifics of any of these requirements - it should just provide the bare minimum required to make them possible. We're trying to push as much into user-land as possible and leave
`rectangle-pack`s responsibility to not much more than answering:

> Given these rectangles that need to be placed, the maximum sizes of the target bins to place them in and some criteria about how to place and how not to place them,
> where can I put all of these rectangles?

## Features

- Minimalist, generic API that pushes as much as possible into user-land.

- Arbitrarily grouping rectangles to ensure that they are placed in the same bin(s).

- Supports three dimensions rectangles through a width + height + depth based API.

- Supports through dimensions rectangles (depth = 1)

- User provided heuristics to grant full control over the packing approach.

## Packing Algorithm

- If necessary a texture will be duplicated in order to 

## To Test

```sh
# Clone the repository
git clone git@github.com:chinedufn/rectangle-pack.git
cd rectangle-pack

# Run tests
cargo test
```

## See Also

- [rectpack2D]
    - Inspired parts of our initial implementation

[rectpack2D]: https://github.com/TeamHypersomnia/rectpack2D

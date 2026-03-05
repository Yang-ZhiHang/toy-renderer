# Rust Path Tracer

The project is a simple *path tracing* renderer implemented in `Rust` language. It demonstrates basic concepts of path tracing, including scene setup, camera configuration, and rendering.

## Features

- [x] Support CPU multithreading.
- [x] Supports rendering the images as `PNG` format.
- [x] Uses BSDF-based microfacet model materials.
- [x] Support to set the shutting time for motion blur effects.
- [x] Support to perform multiple rounds of rendering (iterative render).

## Attention

- In order to avoid numerical issues, the index of refraction and roughness values are clamped to safe ranges in `material.rs`.
- If you wanna make a hollow glass sphere, it's better to set the index of refraction of the inner sphere to reciprocal value (e.g., 1.0 / 1.5) than setting the radius to negative value.
- Currently, the renderer doesn't support to render the shape of light sources directly. You can only see their effects on other objects in the scene.

## References

- [Ray Tracing in One Weekend - Book Series](https://raytracing.github.io/): A popular book that provides a comprehensive introduction to path tracing.
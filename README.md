# Vulkan Render Graph

This is a low-level interface for Vulkan, allowing the user to specify a set a set of render/compute passes and their dependencies. VRG then automatically creates the necessary Vulkan objects, including synchronisation, which is then automatically executed on the GPU. It has a custom math library including vectors, matrices, and quaternions, with 4x4 matrix multiplication accelerated with SIMD.

## Purpose

This is currently a lightweight framework which requires a reasonable understanding of Vulkan to be used. It allows the user to avoid worrying about synchronisation and execution order, but the user must still be mindful of Vulkan's intricacies.

## Architecture

The render graph works by:

1. Building a DAG of render passes and resources
2. Analyzing resource dependencies and lifetimes
3. Optimizing pass order and memory usage
4. Generating necessary synchronization commands
5. Executing the optimized graph

## Files of interest

- `src/layer.rs`: The bulk of the render graph implementation, showcasing dynamic syncronisation object creation.
- `src/compute_pass.rs`: A standard file in VRG, showcasing the implemention of the 'builder' design pattern for ergonomic instantiation.
- `src/util/graph.rs`: A mutable graph datastructure, with an efficient breadth-first search.
- `src/math/mat.rs`: A simple matrix implementation, including common 3D matrices such as projection and view matrices, and SIMD-accelerated 4x4 matrix multiplication.
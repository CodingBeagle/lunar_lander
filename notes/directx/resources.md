# Resources

Resources in DirectX are areas in memory that can be accessed by the Direct3D pipeline.

So resources are the way in which you can provide data to the pipeline, in order to control what should be rendered, and how.

Lifetime of resources:
- You create a resource with one of the Create methods of the ID3D11Device interface.
- You bind the resource to the pipeline using a method on ID3D11DeviceContext.
- You deallocate a resource by calling the Release method of the resource interface.

Direct3D can reference an entire resource or it can reference subsets of a resource.

The *buffer* resource type is defined as a single subresource.

## Types of Resources

Direct3D has several different types of resources:
- Buffers
-- A buffer resource is a collection of fully typed data grouped into elements. You typically use buffers to store things like position vectors, normal vectors, texture coordinates, constant buffers to shaders, vertices, etc...
-- Of buffers, there are the three types: Vertex Buffer, Index Buffer, and Constant Buffer.

### Constant Buffer

A constant buffer allows you to supply shader constants data to the pipeline.

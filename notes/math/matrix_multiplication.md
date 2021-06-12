# Row-Major Form vs Column-Major Form

There's two different notational conventions when it comes to translation matrices and vector forms.

In one (typically used by OpenGL, for example), the translation vectors in a Translation Matrix will be at the right-most column of the 4x4 matrix.
In another one (typically used by DirectX), the translation vectors in a Translation Matrix will be at the bottom row of the 4x4 matrix.

In the first convention, with translation vectors at the right-most column of the matrix, vectors have to be represented as a COLUMN VECTOR when multiplied (and would be multiplied from the right).
This gives you a translation matrix: C = TRSp.

In the second convention, with translation vectors at the bottom row of the matrix, vectors have to be represented as a ROW VECTOR when multiplied.
The order then is: C = pSRT.

Generally, when using graphics APIs, it's important to consider these two properties of your matrices and vectors:

- How does the Graphics API work with vector / matrix memory layout in shaders and so forth? Column-Major or Row-Major?
- How does the Graphics API, or the Math API you use, store vectors and matrices in memory, and which type of translation matrix convention is used?

*NOTICE*

There are two very important concepts that has to be seperated.

- *row-major order* and *column-major order* generally refers to the MEMORY layout of the matrices.
- Whether you build your transform matrices to be multiplied by a ROW vector or a COLUMN vector is a different matter, but this will decide the order of how you multiply your vertex (vector) to your projection / world / model matrix. For row vectors, it has to be multiplied "in front of" the matrix, for column vectors, it has to be multiplied "after" the matrix.
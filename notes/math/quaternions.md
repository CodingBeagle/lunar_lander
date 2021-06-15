# Basic Overview

- When working with quaternions for 3D rotation, we are working with **unit quaternions.**. That is, the quaternions have to be of *unit length*.
- Unit Quaternions can be used to describe any point on a 3D sphere using the formula: **q * p * q^-1** where q^-1 is simply the inverse of quaternion *q*.
  - This formula will rotate a point *p* around an axis by angle 2ø.
  - The result of this formula can be converted to a rotational matrix, which can be used in the graphics pipeline.


Implementation ideas:

- A quaternion can simply be represented by a 4D vector. A quaternion has 4 real components.
- A simply first-starter fuction to create would be one that takes:
  - A 3D vector representing the axis to rotate around (**n**).
  - An angle representing a degree you want to rotate around said axis (**ø**).
- Create the rotation quaternion **q** using the following formula:
  - **q = (sin(ø / 2)n, cos(ø/2))**
  - Then simply use this rotational quaternion **q** in the matrix multiplication
    - **qM**, where **M** is a matrix representing the quaternion formula **qpq***
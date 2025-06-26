# Cross Section Views

Cross section views are common visualization tools in optical design. They represent a three-dimensional arrangement of optical elements such as lenses and mirrors as a two-dimensional line drawing. They are constructed from the paths formed by the intersections between a system's surfaces and a plane known as the cutting plane.

## Cutting Plane Representations

Though in general cutting planes can have any orientation, it is simpler and often desirable to limit the cutting planes to the set of planes that are parallel to the xy, xz, and yz planes in the global coordinate reference frame (CRF). The implicit surface representations for these planes are, respectively,

\\[\begin{eqnarray*}
    z + d &=& 0 \\\\
    y + d &=& 0 \\\\
    x + d &=& 0 
\end{eqnarray*}\\]

where \\( d \\) is some offset of the cutting plane from the origin.

## Surface Intersections with the Cutting Plane

In general, a surface can be positioned and oriented anywhere in 3D space such that none of its principal axes align with those of the cutting plane. The goal is to find the curve that represents the intersection of the cutting plane with the arbitrarily oriented surface.

The approach we will take is as follows:

1. Rotate the equation for the cutting plane into the local CRF for the surface.
2. Substitute the expression for the surface's sag into the rotated equation for the plane.
3. Parameterize the intersection curve into polar coordinates so that it becomes a 1D root-finding problem.
4. Solve the parameterized equation for the radial coordinate.
5. Apply constraints.
6. Transform the curve's sampled points back into Cartesian coordinates.
7. Transform back into the global reference frame.

### Rotate into the Local Coordinate Reference System

The transformation from the global to the local CRF begins by writing the expression for a coordinate in the global CRF in terms of the surface's local CRF.

\\[
    \vec{ x }_ G = R_{GL}^{-1} \vec{x}_L - \vec{ t }_G
\\]

Here, \\( R_{GL}^{-1} \\) is the local-to-global frame rotation matrix and \\( \vec{ t }_G \\) is the position of the surface's apex in the global CRF.


Next we write the cutting plane in its vector notation in the global CRF in terms of its unit normal vector \\( \hat{ n }_G \\).

\\[
    \hat{ n }_G \cdot \vec{ x }_G + d = 0
\\]

Substituting the first expression into the second, we arrive at:

\\[
    \hat{n}_ G \cdot \left( R_{ GL }^{ -1 } \vec{ x }_L - \vec{ t }_G \right) + d = 0
\\]

Carrying through the dot product:

\\[
    \hat{ n }_ G \cdot R_{ GL }^{ -1 } \vec{ x_L } - \hat{n}_G \cdot \vec{t_G} + d = 0
\\]

Now we need to simplify this expression. Regarding the first term, rotation matrices are orthogonal, which means that their inverses are equivalent to their transposes. Additionally, we can apply the identity \\( \vec{ a } \cdot \left( M^\mathrm{ T } \vec{ b }\right) = \left( M a \right) \cdot b \\). This produces the following:

\\[
    \left( R_{GL} \hat{ n }_G \right) \cdot \vec{ x_L } - \hat{ n }_G \cdot \vec{t}_G + d = 0
\\]

The expression \\( R_{GL} \hat{ n }_G \\) is just the cutting plane's unit normal vector rotated into the local CRF, so we can replace it with \\( \hat{ n }_L \\). We can also replace the second term in the equation with the scalar \\( d_L = - \hat{ n }_G \cdot \vec{t}_G + d \\). Upon making these substitutions, we find that have arrived at the implicit vector equation for the cutting plane in the local CRF:

\\[
    \hat{ n }_L \cdot \vec{ x }_L + d_L = 0
\\]

We know \\( R_{GL} \\) and \\( \vec{ t }_G \\) because they are surface properties, and \\( \hat{ n}_G \\) and \\( d \\) are just user inputs that define the cutting plane. This means we have everything we need to find the cutting plane's intersection with the surface.

### Substitute in the Expression for the Surface Sag

Surfaces in the Cherry Ray Tracer are represented by their sag, which in their local coordinate systems is some function \\( z = f (x, y) \\). We will now substitute this expression for \\( z \\) in the equation for the cutting plane expressed in the local CRF to obtain the curve representing the intersection of the cutting plane with the surface.

To do this, we start by expanding the first dot product in the expression for the cutting plane:

\\[\begin{eqnarray*}
    0 &=& \left( R_{GL} \hat{ n }_ G \right) \cdot \vec{ x }_ L - \hat{ n }_ G \cdot \vec{t}_ G + d \\\\
    0 &=& \left( R_{ GL, 1 } \cdot \hat{ n }_ G \right) x_L + \left( R_{ GL, 2 } \cdot \hat{ n }_ G \right) y_L + \left( R_{ GL, 3 } \cdot \hat{ n }_ G \right) z_L + d_L
\end{eqnarray*}\\]

\\( R_{GL, i} \\) represents the i'th row of the rotation matrix above. Substituting the sag for \\( z \\) provides the expression curve.

\\[
    0 =\left( R_{ GL, 1 } \cdot \hat{ n }_ G \right) x_L + \left( R_{ GL, 2 } \cdot \hat{ n }_ G \right) y_L + \left( R_{ GL, 3 } \cdot \hat{ n }_ G \right) f (x_L, y_L) + d_L
\\]

This is a 2D equation! The set of all values \\( (x_L, y_L) \\) that satisfy it is *a curve in the cutting plane.*

### Parameterize the Intersection

Solving this equation for \\( x_L \\) and \\( y_L \\) in general is not easy. Moreover, we will have constraints on their allowed values that represent, for example, the size of a surface's aperture that we will need to apply in the next step. It is not clear how to do that if both \\(x \\) and \\( y \\) are independent variables. 

We can solve both of these problems by parameterizing the surface in polar coordinates. Doing so will transform the problem into a 1D root-finding problem which can be solved numerically. And since the curve is closed, we can sample the entire curve by allowing the polar angle to take on the values \\( 0 \leq \theta < 2 \pi \\). The idea is illustrated below. **TODO**

We use the usual Cartesian-to-polar transformation equations to parameterize the curve:

\\[\begin{eqnarray*}
    x &=& r \cos \theta \\\\
    y &=& r \sin \theta 
\end{eqnarray*}\\]

In polar coordinates, the intersection curve becomes

\\[
    0 =\left( R_{ GL, 1 } \cdot \hat{ n }_ G \right) r \cos \theta + \left( R_{ GL, 2 } \cdot \hat{ n }_ G \right) r \sin \theta + \left( R_{ GL, 3 } \cdot \hat{ n }_ G \right) f (r, \theta) + d_L
\\]

which of course assumes that we can write our surface sag in polar coordinates as well.

### Solve for the Radial Coordinate
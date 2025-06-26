# Defintions and Properties

## Explicit, Implicit and Parametric Representations

### Explicit

Explicit surfaces are defined directly in terms of the points on the surface. They are usually of the form

\\[
    z = f ( x, y )
\\]

An example of an explicit surface is the paraboloid \\( z = x^2 + y^2 \\) which is depicted below. The value \\( z \\) in this case represents a height map above the xy plane. Given any pair of values \\( (x, y ) \\), we can obtain the corresponding \\( z \\) value from this equation.

The advantage to using explicit surface representations is that they provide a mechanism for easily generating points on the surface.

### Implicit

Implicit surfaces are defined by the equation

\\[
    f ( x, y, z ) = 0
\\]

It is easy to determine whether a point lies on an implicitly defined surface: simply plug the coordinates into the expression for the surface and check whether the result equals 0. It is also convenient to check whether a point lies "above" ( \\( f ( x, y, z ) > 0 \\) ) or "below" ( \\( f (x, y, z) < 0 \\) ) an implicit surface. For these reasons, implicit surfaces are preferred for operations such as bounds checking.

## Properties of Surfaces

Surfaces may be flat, curved, or moderately complex. When modeling optical systems, however, they are commonly understood to be **smooth**.

When a ray intersects a surface, its direction may change depending on a few properties of the surface and its surroundings, such as whether it's reflecting or refracting, or the refractive indexes before and after the surface. Among these properties, the surface **normal vector** at the intersection point plays an important role in computing the ray's new direction of travel.

Surfaces can have infinite extent, or they may be constrained to lie within a certain area of space. The **domain** of a surface is the set of all points that lie on the surface.

**Sag** is another important property in optics. Sag is the perpendicular distance between the tangent plane at the surface's apex and the surface itself. It represents the surface as something like a height map.

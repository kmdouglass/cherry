# Conics

A conic surface is defined by the following expression:

\\[
    r^2 - 2 R z + (K + 1) z^2 = 0
\\]

Here, \\( r^2 = x^2 + y ^2 \\) represents a location in the xy plane and \\( z \\) is the distance from this plane to the surface. \\( R \\) is the radius of curvature of the surface at the origin, and \\( K \\) is known as the conic constant[^1]. Different values of \\( K \\) lead to different types of conics, for example:

- \\( K < -1 \\) : hyperbola
- \\( K = -1 \\) : parabola
- \\( -1 < K < 0 \\) : prolate ellipse
- \\( K = 0 \\) : sphere
- \\( K > 0 \\) : oblate ellipse

\\( K = 0 \\), or spherical surfaces, are arguably the most important conics because they are easy to manufacture. We can verify that this expression reduces to the more familiar equation for a sphere by rewriting \\( z^2 - 2 R z = (z - R)^2 - R^2 \\), after which the above expression becomes

\\[\begin{eqnarray*}
    r^2 + ( z - R )^2 &=& R^2 \\\\
    x^2 + y^2 + ( z - R )^2 &=& R^2
\end{eqnarray*}\\]

which is the equation for a spherical surface of radius \\( R \\) centered at \\( z = R \\).

## Surface Sag of Conics

The surface sag of a conic is the solution to the conic equation for \\( z \\). We derive it by applying the quadratic formula:

\\[\begin{eqnarray*}
    z &=& \frac{2 R \pm \sqrt{ 4 R^2 - 4 ( K + 1 ) r^2} }{ 2 (K + 1) } \\\\
    &=& \frac{R \pm \sqrt{ R^2 - ( K + 1) r^2 } }{ K + 1 }
\end{eqnarray*}\\]

This equation becomes problematic in two scenarios:

1. When the conic is a parabola, \\( K = -1 \\) and the denominator becomes 0.
2. When the surface is flat, i.e. \\( R = \infty \\).

To solve the first problem, we rationalize the numerator by multiplying by its conjugate.

\\[\begin{eqnarray*}
    z &=& \frac{R \pm \sqrt{ R^2 - ( K + 1) r^2 } }{ K + 1 } \left( \frac{ R \mp \sqrt{ R^2  - ( K + 1 ) r^2 } } { R \mp \sqrt{ R^2  - ( K + 1 ) r^2 } } \right) \\\\
    &=& \frac{ R^2 - R^2  + ( K + 1 ) r^2 }{ ( K + 1 ) \left( R \mp \sqrt{ R^2  - ( K + 1 ) r^2 } \right)} \\\\
    &=& \frac{ r^2 }{ R \mp \sqrt{ R^2 - ( K + 1) r^2} }
\end{eqnarray*}\\]

Now, we would ideally like to have a sag that is continuous as we change the curvature from negative to posivite and vice-versa without any numerical issues that come from numbers approaching infinity. We can avoid these if we define the curvature \\( C \equiv 1 / R \\) and rewrite the above expression using it instead:

\\[\begin{eqnarray*}
    z &=& \frac{ r^2 / R }{ 1 \mp \sqrt{ 1 - ( K + 1) r^2 / R ^2 } } \\\\
    &=& \frac{ r^2 C }{ 1 \mp \sqrt{ 1 - ( K + 1) (rC)^2 } }
\end{eqnarray*}\\]

Finally, we need to address the fact that there is a \\( \mp \\) in the denominator. Strictly speaking, both plus and minus signs produce valid solutions for the conic curve. However, in our geometry surface sag is the perpendicular distance from the xy plane to the first point of intersection of the curve. This means we need to take the solution with the smaller z-value, i.e. we use the plus sign. Taking the plus sign also avoids a division by zero in the case of a parabola, \\( K = -1 \\).

The surface sag of a conic is therefore

\\[
    z = \frac{ r^2 C }{ 1 + \sqrt{ 1 - ( K + 1) (rC)^2 } }
\\]

[^1]: [https://en.wikipedia.org/wiki/Conic_constant](https://en.wikipedia.org/wiki/Conic_constant)

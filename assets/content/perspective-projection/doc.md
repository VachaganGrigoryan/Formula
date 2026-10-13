# Perspective projection

## Overview

Perspective projection turns a 3D point into a 2D screen position the way a camera or an eye does: everything is seen from a single point, so objects further away look smaller and parallel lines running away from the viewer converge toward a vanishing point. The whole effect comes from one operation, **dividing by depth**.

It is the last geometric step of almost every 3D renderer, from GPU pipelines to a hand-written 3D scatter plot, and the basic camera model in computer vision. The app projects a cube, a floor grid and 70 random points; an inset shows one ray from above.

## Formula

```math
x' = f · x / z + c_{x}
y' = f · y / z + c_{y}
```

| Symbol | Meaning |
| --- | --- |
| x, y, z | the point in camera space: z is the depth along the viewing direction |
| f | focal length: distance from the eye to the image plane, in pixels |
| `cx`, `cy` | the principal point, usually the center of the screen |
| x', y' | the projected screen position |

Before projecting, each scene point is moved into camera space. The app rotates it by a yaw angle ψ around the vertical axis and a pitch angle φ around the horizontal axis, then pushes it a distance d in front of the camera:

```math
x_{1} = x cos ψ + z sin ψ,   z_{1} = −x sin ψ + z cos ψ
y_{2} = y cos φ − z_{1} sin φ,   z_{2} = y sin φ + z_{1} cos φ
p_{cam} = (x_{1}, y_{2}, z_{2} + d)
```

Screens usually grow downward, so the app uses `y' = -f * y / z + cy`, which keeps "up" pointing up.

## Intuition

Put the eye at the origin, looking along +z, and hold a sheet of glass (the image plane) at distance f in front of it. A point at (x, z) sends a ray to the eye; where that ray crosses the glass is its image. The eye, the foot of the glass and the crossing point form a small right triangle; the eye, the point and its foot on the z axis form a big one. They are **similar triangles**, so

```math
x' / f = x / z,   so   x' = f · x / z
```

This is exactly the inset "seen from above". Two consequences fall out at once:

- **Twice as far is half as big.** Doubling z halves x' and y'. That is why the app draws far points smaller and closer to the center.
- **Focal length is zoom.** Doubling f doubles every screen offset. A short f gives a wide field of view, with half-angle atan(w / 2f) for a screen of width w; a long f gives a narrow, telephoto view.

The **dolly zoom** combines both: move the camera back (larger d) while lengthening f in proportion. The object at the center keeps its size, because f / d stays constant, but the depth differences inside it matter less and less, so the cube flattens toward an orthographic look.

## Algorithm

1. Generate or load the scene points (here: cube corners, floor grid endpoints, the scatter).
2. Rotate each point by yaw, then pitch, and add the camera distance d to z.
3. Discard any point with z at or below a small near distance (the app uses 0.05); dividing by zero or a negative depth would flip it through the eye.
4. Project: `x' = f * x / z + cx` and `y' = -f * y / z + cy`.
5. Sort the points by depth, far first, and draw them in that order so near points cover far ones (the painter's algorithm). Scale each marker by f / z and fade it with depth.
6. Draw edges between projected endpoints; skip an edge if either end was culled.

```
for p in scene:
    q = rotate(p, yaw, pitch); q.z += d
    if q.z > NEAR: draw(f*q.x/q.z + cx, -f*q.y/q.z + cy, size = k*f/q.z)
```

## Complexity

| Task | Time | Space |
| --- | --- | --- |
| project one point | O(1): two divisions (or one reciprocal) and a few multiplications | O(1) |
| project n points | O(n) | O(n) |
| depth-sort for drawing | O(n log n) | O(n) |
| GPU form: 4x4 matrix times vector, then divide by w | O(1) per vertex | O(1) |

## Parameters in the app

- **Focal length f**: 0.3 to 2.5 times the stage's shorter side. Larger is more zoomed in and flatter.
- **Camera distance**: how far the camera sits from the center of the scene, 2.2 to 12. Close up, perspective is strong and the near face of the cube looks much larger than the far one.
- **Spin / Stop spinning**: turns the scene around the vertical axis (the yaw).
- **Dolly zoom**: jumps between a close camera with a short lens and a far camera with a long lens (f = 0.25 d), so the cube keeps its size but loses or gains its perspective.
- **Drag on the stage**: horizontal motion changes yaw, vertical motion changes pitch (limited to about ±80°).
- The highlighted point is the one traced in the inset, which shows its ray from the eye, the image plane at f, and the crossing x'.

## Numerical notes and pitfalls

- **Points behind the camera.** For z < 0 the division flips the point to the opposite side of the screen; at z = 0 it explodes. Always clip against a near plane first.
- **Clipping lines.** Culling whole edges whose endpoint is behind the near plane (as the app does) makes long lines vanish abruptly. A renderer instead clips the segment at the near plane, then projects the clipped endpoint.
- **Interpolation is not linear on screen.** Colors, texture coordinates and depth cannot be lerped linearly in screen space. Lerp a / z and 1 / z instead and divide (perspective-correct interpolation).
- **Depth precision.** GPUs store a value proportional to 1 / z, which spends most of its precision near the camera. Pushing the near plane too close causes z-fighting far away.
- **Painter's algorithm limits.** Sorting by depth works for points and small sprites, but intersecting or cyclically overlapping polygons need a depth buffer.
- **Units and aspect.** f must be in the same units as the screen offsets. If pixels are not square, or the viewport is not square, use separate focal lengths `fx` and `fy` for the two axes.
- **y direction.** Mixing a y-up camera with a y-down screen without the sign flip renders the world upside down.

## Applications

- Real-time and offline 3D graphics: the projection matrix and the "divide by w" in every GPU pipeline.
- Computer vision: the pinhole camera model with intrinsic matrix K, camera calibration, structure from motion, augmented reality.
- Data visualization: 3D scatter plots, bar charts with depth, globes, terrain and 3D network layouts.
- Cinematography: choosing lens and camera distance, and the dolly zoom effect.

## History

The camera obscura, the physical version of the pinhole model, was described by the Chinese philosopher Mozi in the 5th century BC and analyzed by Ibn al-Haytham (Alhazen) in his *Book of Optics* around 1021. Linear perspective in painting was demonstrated by Filippo Brunelleschi in Florence around 1415 and written down by Leon Battista Alberti in *De pictura* (1435); Piero della Francesca and Albrecht Dürer later published geometric constructions and drawing machines. Girard Desargues (1639) and Jean-Victor Poncelet (1822) turned the ideas into projective geometry, and Möbius and Plücker introduced homogeneous coordinates around 1827. In computing, Lawrence Roberts' 1963 MIT thesis used homogeneous coordinates for perspective, the form GPUs still use. The word *perspective* comes from the Latin *perspicere*, "to see through". The dolly zoom was first used by Irmin Roberts for Alfred Hitchcock's *Vertigo* (1958).

## References

- Wikipedia, "3D projection" and "Pinhole camera model".
- Richard Hartley and Andrew Zisserman, "Multiple View Geometry in Computer Vision", 2nd edition, Cambridge University Press, 2004.
- John F. Hughes, Andries van Dam et al., "Computer Graphics: Principles and Practice", 3rd edition, Addison-Wesley, 2013.
- Lawrence G. Roberts, "Machine Perception of Three-Dimensional Solids", MIT PhD thesis, 1963.
- Leon Battista Alberti, "De pictura" (On Painting), 1435.

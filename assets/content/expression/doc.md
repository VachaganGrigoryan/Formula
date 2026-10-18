# Expression plotter

## Overview

The expression plotter draws functions you type yourself. Each row is one definition: an explicit function `y = a sin(bx)`, a polar curve `r = 1 + cos(3θ)`, or a parametric curve `(cos t, sin t)`. Every letter that is not the curve's own variable becomes a parameter with its own slider, so you can watch how a family of curves changes as a coefficient moves.

It is the same idea as a graphing calculator: the text is parsed once into an expression tree, then the tree is evaluated at many points and the points are joined into a curve, with the line broken wherever the function is undefined or jumps across an asymptote.

## Formula

```math
y = f(x)                 explicit row, sampled across the view
r = f(θ),  x = r cos θ,  y = r sin θ      polar row, θ from 0 to 2π
(x, y) = (g(t), h(t))    parametric row, t from 0 to 2π
```

| Symbol | Meaning |
| --- | --- |
| x | the variable of an explicit row (`f(t) = ...` uses t instead) |
| θ | the angle of a polar row; type `theta` or `θ` |
| t | the parameter of a parametric row |
| a, b, k, ... | free parameters, one slider each, starting at 1 in the range −10 to 10 |
| π, e | the constants; type `pi` or `π`, and `e` |

Functions: `sin cos tan asin acos atan sqrt abs exp ln log floor ceil` take one argument and `min max` take two. `ln` is the natural logarithm and `log` is base 10.

## Intuition

Letters are single-letter variables, as on paper: `ab` means a times b, `2xy` means 2 times x times y, and `k_r` or `a_1` is one subscripted name. Function names, Greek letter names (`theta`, `alpha`, `omega`, ...) and `pi` are read as whole words, so `sinx` is sin(x) and `pix` is π times x.

Multiplication can be left out wherever it would be on paper: `2x`, `3(x + 1)`, `(x + 1)(x − 1)`, `2 sin x`. A function written without brackets takes the product that follows it: `sin 2x` is sin(2x), and `sin x cos x` is sin(x) times cos(x). Powers bind tightest and group from the right, and a minus sign in front applies after the power: `-x^2` is −(x²), `2^3^2` is 2⁹ and `2^-1` is one half.

## Algorithm

1. **Lex**: split the text into numbers, names and operators. A run of letters is cut into the longest known word or a single letter at each position.
2. **Parse** with a Pratt parser (top-down operator precedence): sums are made of products, products of signed factors, factors of powers, and powers of atoms (numbers, names, calls, brackets). Two atoms side by side are an implicit product. The depth is capped at 256 so a pathological input cannot overflow the stack.
3. **Classify** the line by its head: `y =`, `f(v) =`, `r =`, a bracketed pair, or a bare expression (read as `y = ...`).
4. **Collect parameters**: every name except the curve's variable and the constants gets a slider. Editing a row keeps the values of parameters that are still used.
5. **Sample** an explicit row at about one point per two screen points, then subdivide any interval whose midpoint is more than half a point off the chord (up to 64 pieces). A step that is not finite, or that jumps by more than five view heights without passing through the midpoint, starts a new line. Polar and parametric rows are sampled at 720 steps with the same breaks.

## Complexity

Parsing is linear in the length of the text. Evaluating a tree costs one operation per node. An explicit row takes about w/2 evaluations for a view w points wide, and at most 64 times that near asymptotes and sharp turns. A curve row takes 721 evaluations per frame.

## Parameters in the app

- **Rows**: up to six; type into a row to change it, *Remove* deletes it and *Add row* adds a blank one. Rows 1 to 3 are solid in three colors, rows 4 to 6 repeat the colors dashed.
- **Errors** are shown under the row with a caret at the position, and listed on the stage. The other rows keep plotting.
- **θ from / to** and **t from / to** set the range of a polar or parametric row.
- **Sliders**: one per parameter, with editable minimum and maximum.
- **Stage**: drag to pan, scroll or pinch to zoom at the pointer, hover to read x, y and each explicit row's value. *Reset view* returns to the origin.
- **Command line**: `plot r = 1 + cos(3θ)` places a new plotter with that row.
- **Code export** writes your rows and parameter values into the Python, TypeScript and Rust programs.

## Numerical notes and pitfalls

- Division by zero gives infinity and `sqrt(-1)` or `ln(-1)` give NaN; those points are skipped, so `1/x` draws two branches and `sqrt(x)` starts at zero.
- Asymptotes are detected from the samples. A function that jumps by less than five view heights, such as `floor(x)` at a normal zoom, is drawn with short vertical connecting lines, as most plotters do.
- Very large values are clamped to ten view heights beyond the visible area, so a steep curve leaves the view instead of producing enormous coordinates.
- `e` is always the constant, never a parameter. `1e5` is read as 1 times e times 5 and reports a missing operator; write `10^5`.
- Trigonometric functions use radians.

## Applications

- Exploring families of curves: how a, b and c move a parabola, how k changes the petals of a rose curve.
- Checking homework and derivations by plotting both sides of an identity.
- Prototyping easing curves, signal shapes and parametric paths before writing code, then exporting them.
- Teaching implicit multiplication and operator precedence, since the parser follows the same rules as written math.

## History

Vaughan Pratt described top-down operator precedence parsing in 1973 as a simple way to give each operator a binding power; it became the standard way to hand-write expression parsers, popularised again by Douglas Crockford's JavaScript parser in 2007. Graphing calculators such as the Casio fx-7000G (1985) put function plotting in students' hands, and web tools like Desmos (2011) made typed, slider-driven exploration of formulas common in classrooms.

## References

- Vaughan R. Pratt, "Top Down Operator Precedence", Proceedings of the 1st ACM Symposium on Principles of Programming Languages, 1973.
- Douglas Crockford, "Top Down Operator Precedence", in *Beautiful Code*, O'Reilly, 2007.
- Desmos Graphing Calculator user guide: https://help.desmos.com/

# Line of best fit

## Overview

Given a scatter of points `(x_i, y_i)`, **simple linear regression** finds the straight line `y = m x + b` that best predicts `y` from `x`. "Best" is defined by **least squares**: the line makes the sum of squared vertical misses, the **residuals**, as small as possible. That choice has a closed-form answer computed in linear time, and the resulting line always passes through the mean point of the data.

It is the most widely used model in statistics and the trend line in every charting tool. Alongside the line comes **R²**, the coefficient of determination, which reports what share of the variation in `y` the line explains. The app lets you drag, add and remove points and see the line, the residuals and their literal squares update.

## Formula

```math
y = m x + b
m = Σ (x_{i} − μ_{x})(y_{i} − μ_{y}) / Σ (x_{i} − μ_{x})^{2}
b = μ_{y} − m μ_{x}
SSE = Σ (y_{i} − m x_{i} − b)^{2}
R^{2} = 1 − SSE / Σ (y_{i} − μ_{y})^{2}
```

| Symbol | Meaning |
| --- | --- |
| `(x_i, y_i)` | the i-th data point, for i = 1 … n |
| `μx`, `μy` | the means of x and of y (often written with a bar) |
| `m` | slope: change in predicted y per unit of x |
| `b` | intercept: predicted y at x = 0 |
| `y_i − (m x_i + b)` | the residual of point i (positive above the line) |
| `SSE` | sum of squared residuals, the quantity being minimized |
| `R²` | coefficient of determination, between 0 and 1 for this fit |

The numerator of `m` is n times the covariance of x and y; the denominator is n times the variance of x. So `m = cov(x, y) / var(x)`, which also equals `r · σy / σx` with `r` the Pearson correlation. For a straight-line fit, `R² = r²`.

## Intuition

Think of each residual as a vertical spring pulling the line toward its point, with energy proportional to the squared stretch. The line settles where the total energy, `SSE`, is lowest. In the app the *Squared errors* option draws each squared residual as an actual square: least squares finds the line with the smallest total area of squares.

### Derivation

`SSE(m, b)` is a bowl-shaped quadratic in the two unknowns, so its minimum is where both partial derivatives vanish:

- `∂SSE/∂b = −2 Σ (y_i − m x_i − b) = 0`: the residuals sum to zero, which gives `b = μy − m μx`. The line passes through `(μx, μy)`.
- `∂SSE/∂m = −2 Σ x_i (y_i − m x_i − b) = 0`: the residuals are uncorrelated with x. Substituting `b` and centering on the means gives the formula for `m`.

### Why vertical, and why squares

The residuals are vertical because the goal is predicting `y` from `x`; minimizing perpendicular distances is a different method (total least squares). Squaring makes the problem smooth with a unique closed-form answer, and, when the noise is Gaussian, least squares is also the maximum-likelihood estimate. The price is sensitivity to outliers: a point twice as far away counts four times as much.

## Algorithm

1. If there are fewer than two points, or every `x` is the same, stop: the slope is undefined.
2. Compute the means `μx` and `μy`.
3. Accumulate `Sxy = Σ (x_i − μx)(y_i − μy)` and `Sxx = Σ (x_i − μx)²`.
4. Set `m = Sxy / Sxx` and `b = μy − m μx`.
5. For R², accumulate `SST = Σ (y_i − μy)²` and `SSE = Σ (y_i − m x_i − b)²`, then `R² = 1 − SSE / SST` (define it as 1 when `SST = 0`).

```
mx, my = mean(xs), mean(ys)
sxy = sum((x - mx) * (y - my))
sxx = sum((x - mx) ** 2)
m = sxy / sxx
b = my - m * mx
```

## Complexity

| Operation | Time | Space |
| --- | --- | --- |
| fit n points (two passes) | O(n) | O(1) |
| add or remove one point with running sums | O(1) | O(1) |
| predict one value | O(1) | O(1) |
| R² and SSE | O(n) | O(1) |

## Parameters in the app

- **Noise** (0 to 3): spread of the generated points around a hidden true line; changing it draws new points.
- **New data**: a fresh random data set with a new hidden slope.
- **Residuals**: draws each point's vertical miss as a segment to the line.
- **Squared errors**: draws each squared residual as a square; the fit minimizes their total area.
- Mouse: drag a point to move it, click empty space to add one, right-click a point to remove it.
- The readout shows the fitted equation, R², the sum of squared residuals and n. Dashed lines mark `μx` and `μy`; the ring is the mean point the line passes through.

## Numerical notes and pitfalls

- **Use centered sums.** The textbook shortcut `m = (nΣxy − ΣxΣy) / (nΣx² − (Σx)²)` subtracts large, nearly equal numbers and loses precision when x is far from zero (dates, timestamps). Subtract the means first, or use Welford-style running updates.
- **Vertical data.** When all x are equal, `Sxx = 0` and the slope is undefined; the app shows a message instead of a line.
- **Outliers.** One far-off point can swing the line. Look at the residuals, and consider robust alternatives such as Theil–Sen or Huber regression.
- **R² is not validation.** A high R² does not show that a straight line is the right model or that x causes y; Anscombe's quartet has four very different data sets with the same line and R². Always plot the data.
- **Direction matters.** Regressing x on y gives a different line from regressing y on x unless the correlation is perfect.
- **Errors in x.** Least squares assumes x is measured exactly. With noise in both coordinates the slope is biased toward zero; use Deming or total least squares.
- **Extrapolation.** The line is only supported over the range of the data; predictions far outside it are guesses.

## Applications

- Trend lines on scatter plots and time series, and "y rises 0.6 per unit of x" annotations.
- Calibration curves for sensors and lab instruments.
- Quick forecasts and baselines before fitting more complex models.
- Finance: a stock's beta is the slope of its returns regressed on market returns.
- Physics labs: estimating constants such as a spring's stiffness from measurements.

## History

The method of least squares was published by Adrien-Marie Legendre in 1805 for fitting comet orbits. Carl Friedrich Gauss published it in 1809, connected it to the normal distribution, and stated that he had used it since 1795, including to predict where the dwarf planet Ceres would reappear in 1801. The word *regression* comes from Francis Galton, who in 1886 observed that children of unusually tall parents tend to be closer to average height, a "regression towards mediocrity". Karl Pearson formalized the correlation coefficient in the 1890s, and Sewall Wright introduced the term *coefficient of determination* in 1921.

## References

- A.-M. Legendre, *Nouvelles méthodes pour la détermination des orbites des comètes*, 1805.
- C. F. Gauss, *Theoria motus corporum coelestium*, 1809.
- F. Galton, "Regression towards mediocrity in hereditary stature", 1886.
- F. J. Anscombe, "Graphs in statistical analysis", The American Statistician, 1973.
- D. Freedman, R. Pisani and R. Purves, *Statistics*, chapters on regression.
- Wikipedia, "Simple linear regression" and "Coefficient of determination".

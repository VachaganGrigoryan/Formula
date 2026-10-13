# Min-max normalization

## Overview

Min-max normalization rescales a list of numbers so that the smallest becomes 0, the largest becomes 1, and everything else lands proportionally in between. It removes the units and the offset of the raw data and keeps only the *relative* position of each value inside its own range.

It is the first step of almost every chart. A bar's height, a dot's position on a scatter plot, a cell's intensity in a heatmap and a bubble's size are all "where does this value sit between min and max", multiplied by the pixels available. In machine learning the same transform, called feature scaling, puts features measured in meters, dollars and counts on a common 0 to 1 scale before training.

## Formula

```math
X_{norm} = (X − X_{min}) / (X_{max} − X_{min})
height = X_{norm} · H_{pixels}
X_{scaled} = a + X_{norm} · (b − a)
```

| Symbol | Meaning |
| --- | --- |
| `X` | one raw value |
| `X_min`, `X_max` | the smallest and largest value in the dataset |
| `X_norm` | the normalized value, between 0 and 1 |
| `H_pixels` | the height (or width) of the drawing area |
| `a`, `b` | an arbitrary target range, such as -1 to 1 |

The first line maps the data onto 0 to 1; the second turns that fraction into pixels; the third is the general form for any target range `[a, b]`.

## Intuition

Subtracting `X_min` slides the data so that the smallest value sits at zero. Dividing by the range X_max − X_min shrinks or stretches it so that the largest value lands exactly on 1. The transform is linear, so it never changes the *order* of the values or the *ratios of the gaps* between them: if one value is twice as far from the minimum as another before scaling, it still is afterwards.

It is the inverse of linear interpolation. A lerp goes from a fraction `t` to a value between two endpoints; min-max normalization goes from a value back to its fraction `t` between the dataset's own endpoints. Chaining the two gives a **remap** from one range into another, which is exactly what a chart axis does.

The weakness is visible in the app: the whole scale is pinned to the two most extreme values. Push one value far above the rest and the denominator grows, so every other value is divided by a large number and squeezed toward 0.

## Algorithm

1. Scan the data once to find `X_min` and `X_max`.
2. Compute the range span = X_max − X_min.
3. If `span` is 0 (all values equal), return 0 for every value (or 0.5, by convention).
4. Otherwise map each value to (X − X_min) / span.
5. To draw, multiply by the available pixel height, and flip the y axis if the screen's origin is at the top.

```
lo, hi = min(values), max(values)
span = hi - lo
for each v in values:
    norm = (v - lo) / span   if span > 0 else 0
    bar_height = norm * H
```

## Complexity

| Step | Time | Space |
| --- | --- | --- |
| find min and max | O(n) | O(1) |
| normalize every value | O(n) | O(n) for the output, O(1) in place |
| one new value, bounds fixed | O(1) | O(1) |

A streaming version keeps a running min and max, but every time a bound changes all previously normalized values become stale.

## Parameters in the app

- **Outlier (Aug)**: the raw value of August, from 0 to 500. Drag it up to watch the other eleven bars collapse toward the baseline; drag it below the others and it becomes the new minimum instead.
- **New data**: draws twelve new monthly values around a seasonal curve.
- **Reset outlier**: puts August back to its generated value.
- **Raw values axis**: the data on its own scale, with the minimum and maximum labeled.
- **Normalized axis**: the same points after the transform; the lines show where each raw value lands between 0 and 1.
- Hover a bar to see its full calculation and its height in pixels.

## Numerical notes and pitfalls

- **Division by zero.** A constant dataset has `X_max = X_min`. Guard the division and pick a convention (0, 0.5, or "draw nothing").
- **Outliers dominate.** One extreme value compresses everything else into a thin band. Clip to percentiles (for example the 1st and 99th) first, or use a robust scaler based on the median and interquartile range, or a log scale.
- **New data out of range.** In machine learning the min and max are learned on the training set. Test values outside that range map below 0 or above 1; clamp them or accept the extrapolation, but never refit the scaler on test data.
- **Not standardization.** Z-score scaling (X − μ) / σ centers on the mean and divides by the standard deviation; it has no fixed bounds. Min-max gives a fixed range but depends entirely on two samples.
- **NaN and infinity.** A single NaN poisons `min` and `max` in many libraries; filter missing values first.
- **Precision.** When the values are huge but close together (timestamps in nanoseconds, say), subtract the minimum before converting to single precision, or the differences are lost.
- **Bar charts should usually start at zero.** Normalizing with `X_min` as the baseline exaggerates small differences; many charts use 0 instead of `X_min` for the lower bound.

## Applications

- Chart axes: bar heights, scatter positions, line charts and sparklines.
- Heatmaps and choropleth maps: mapping a value to a position on a color ramp.
- Bubble and symbol sizes in data visualization.
- Feature scaling for k-nearest neighbors, k-means, neural networks and gradient descent, where features on large scales would otherwise dominate distances and gradients.
- Image processing: contrast stretching maps the darkest pixel to black and the brightest to white.
- Audio: peak normalization scales a recording so its loudest sample reaches full scale.

## History

Rescaling a quantity by its range is as old as graph paper; William Playfair's bar and line charts of the 1780s already mapped values onto a fixed drawing height. The name *min-max normalization* comes from data mining and machine learning in the 1990s, where it was listed alongside z-score and decimal scaling as standard preprocessing steps. Visualization libraries such as D3 call the same operation a *linear scale* with a domain and a range.

## References

- Jiawei Han, Micheline Kamber and Jian Pei, *Data Mining: Concepts and Techniques*, section on data transformation and normalization.
- Wikipedia, "Feature scaling".
- scikit-learn documentation, "MinMaxScaler" and "Preprocessing data".
- Mike Bostock, D3.js documentation, "d3-scale: linear scales".
- Edward Tufte, *The Visual Display of Quantitative Information*, 1983.

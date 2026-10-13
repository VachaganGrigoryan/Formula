# Relative luminance

## Overview

**Relative luminance** `Y` measures how bright a color looks, on a scale from 0 for black to 1 for white. Equal amounts of red, green and blue light do not look equally bright: the eye is far more sensitive to green than to red, and least sensitive to blue. Relative luminance accounts for this by weighting the three channels, after undoing the sRGB gamma curve so that the weights apply to actual light.

Its everyday use is legibility. The Web Content Accessibility Guidelines (WCAG) define the **contrast ratio** between two colors from their luminances, and the same comparison decides whether a label on a colored background should be black or white. The app labels every cell of a heatmap with its `Y` and picks the text color with the higher contrast.

## Formula

```math
c_{lin} = c / 12.92,   if c ≤ 0.04045
c_{lin} = ((c + 0.055) / 1.055)^{2.4},   otherwise
Y = 0.2126 R_{lin} + 0.7152 G_{lin} + 0.0722 B_{lin}
contrast = (Y_{light} + 0.05) / (Y_{dark} + 0.05)
Y_{switch} = √(1.05 × 0.05) − 0.05 ≈ 0.179
```

| Symbol | Meaning |
| --- | --- |
| `c` | one sRGB channel, the 8-bit value divided by 255 (0 to 1) |
| `c_lin` | that channel converted to linear light |
| `R_lin`, `G_lin`, `B_lin` | the three linearized channels |
| `Y` | relative luminance, 0 (black) to 1 (white) |
| `Y_light`, `Y_dark` | the larger and smaller luminance of the two colors compared |
| `contrast` | WCAG contrast ratio, from 1:1 (identical) to 21:1 (black on white) |
| `Y_switch` | the background luminance at which black and white text contrast equally |

The weights sum to 1, so white has `Y = 1`. Above `Y_switch` black text has more contrast; below it, white text does.

## Intuition

### Why weights

Human daylight vision follows the luminosity function, a sensitivity curve peaking in the green at about 555 nm. Integrating the sRGB red, green and blue primaries against that curve gives their brightness contributions: green 71.5%, red 21.3%, blue 7.2%. Pure blue therefore has `Y ≈ 0.07`, and pure green `Y ≈ 0.72`, even though both are "full strength".

### Why linearize first

sRGB values are stored gamma-encoded: the numbers are spaced evenly for perception, not for light. Mid-gray 128 means only about 21.6% of the light of white. Light from the three channels adds linearly, so the weights must be applied to linear-light values. Skipping the step overstates midtones (gray 128 would get `Y ≈ 0.5` instead of 0.216) and moves the black/white switch to the wrong place. The app's *Linearize* option lets you compare both.

### Where the text color flips

Black text has contrast `(Y + 0.05) / 0.05` against a background of luminance `Y`; white text has `1.05 / (Y + 0.05)`. Setting them equal gives `(Y + 0.05)² = 1.05 × 0.05`, so the switch is at `Y ≈ 0.179`. That is far below 0.5: on mid-gray, yellow, cyan and most saturated light colors, black text wins. The constant 0.05 models flare, the ambient light reflected from a real screen, which keeps very dark colors from having infinite contrast.

## Algorithm

1. Divide each 8-bit channel by 255.
2. Linearize each channel with the piecewise sRGB curve.
3. Compute `Y = 0.2126 R + 0.7152 G + 0.0722 B` on the linear values.
4. For two colors, divide `(lighter Y + 0.05)` by `(darker Y + 0.05)`.
5. For a label, compute the contrast with black and with white and pick the larger, which is the same as testing `Y ≥ 0.179`.

```
lin(c) = c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4
Y = 0.2126*lin(r/255) + 0.7152*lin(g/255) + 0.0722*lin(b/255)
text = black if (Y + 0.05) / 0.05 >= 1.05 / (Y + 0.05) else white
```

## Complexity

| Operation | Time | Space |
| --- | --- | --- |
| luminance of one color | O(1): 3 linearizations, 3 multiplications | O(1) |
| with a 256-entry lookup table for `lin` | O(1), no `pow` calls | O(256) |
| contrast ratio of two luminances | O(1) | O(1) |
| labeling n heatmap cells | O(n) | O(1) extra |

## Parameters in the app

- **Heatmap**: 12 hues by 5 rows of saturation and value, light to dark. Each cell shows its `Y`, written in black or white, whichever contrasts more.
- **Click a cell** (or use the **Color** picker) to inspect a color in the panel below; hovering outlines a cell.
- **Linearize sRGB first (WCAG)**: on, the channels are linearized as WCAG specifies; off, the weights are applied to the encoded values, a common shortcut that misjudges midtones.
- **Naive: always white text**: labels every cell in white, to show where that fails.
- The panel shows the swatch with both text colors and their contrast ratios, a stacked bar of the red, green and blue contributions to `Y` with the switch point at 0.179 marked, and the formula with the selected color's numbers.

## Numerical notes and pitfalls

- **Threshold constant.** WCAG 2 quotes 0.03928 for the linear segment, from an early sRGB draft; the sRGB standard uses 0.04045. No 8-bit value lies between them (10/255 is below both, 11/255 above), so results are identical.
- **Luma is not luminance.** Video luma `Y' = 0.299 R' + 0.587 G' + 0.114 B'` (Rec. 601) is computed on gamma-encoded values with different weights. It is fine for compression, not for contrast checks.
- **Lightness is not luminance.** HSL lightness ignores hue entirely: pure yellow and pure blue both have L = 50%, but luminances of 0.93 and 0.07.
- **Do not threshold at 0.5.** The black/white switch is at `Y ≈ 0.179`; a 0.5 cut puts white text on many mid-tones where black reads better.
- **Transparency.** Composite a translucent color over its background before measuring it.
- **No rounding up.** A ratio of 4.48:1 fails a 4.5:1 requirement.
- **Limits of the model.** WCAG 2 contrast ignores font weight and polarity and misjudges some dark pairs; the APCA method proposed for WCAG 3 addresses this.

## Applications

- Choosing black or white labels on heatmap cells, bars, pie slices, tags and avatars with data-driven colors.
- Checking text and UI colors against WCAG: 4.5:1 for body text, 3:1 for large text and UI components, 7:1 for the enhanced level.
- Building accessible palettes and dark-mode themes in design systems.
- Converting color images to grayscale that preserves perceived brightness.

## History

The CIE standardized the photopic luminosity function in 1924, and its 1931 XYZ color space defines `Y` as luminance. The weights 0.2126, 0.7152 and 0.0722 are the Y row of the matrix converting the primaries of ITU-R Recommendation BT.709 (1990), the HDTV standard, with a D65 white point, to XYZ. Hewlett-Packard and Microsoft proposed sRGB in 1996 with the same primaries and a gamma-like transfer curve; it was standardized as IEC 61966-2-1 in 1999. WCAG 2.0 (2008) adopted *relative luminance* and the contrast ratio, basing the 4.5:1 minimum on an ISO 9241-3 ratio of 3:1, raised to compensate for the reduced contrast sensitivity of people with moderately low vision.

## References

- W3C, "Web Content Accessibility Guidelines (WCAG) 2.1", definitions of relative luminance and contrast ratio, and "Understanding Success Criterion 1.4.3: Contrast (Minimum)".
- IEC 61966-2-1:1999, "Default RGB colour space: sRGB".
- ITU-R Recommendation BT.709, "Parameter values for the HDTV standards for production and international programme exchange".
- C. Poynton, *Digital Video and HD: Algorithms and Interfaces*, chapters on luminance and luma.
- Wikipedia, "Relative luminance" and "sRGB".

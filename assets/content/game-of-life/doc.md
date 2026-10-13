# Conway's Game of Life

## Overview

The Game of Life is a cellular automaton: a grid of square cells, each either alive or dead, that updates in discrete generations. A cell's next state depends only on its own state and on how many of its eight neighbors are alive. There are no players and no randomness after the start; you choose an initial pattern and watch it evolve.

Four tiny rules produce a whole zoo of behavior: still lifes that never change, oscillators that blink with a fixed period, gliders and spaceships that crawl across the grid, guns that fire gliders forever, and chaotic soups that churn for thousands of generations before settling. Life is the standard example of *emergence*, complex structure arising from simple local rules, and it is Turing complete: with enough room it can compute anything a computer can.

## Formula

```math
n(x, y) = Σ s_{t}(x + dx, y + dy),   dx, dy = −1, 0, 1,   (dx, dy) ≠ (0, 0)
s_{t+1} = 1  if  n = 3,  or  if  s_{t} = 1 and n = 2
s_{t+1} = 0  otherwise
```

| Symbol | Meaning |
| --- | --- |
| `s_t(x, y)` | state of the cell at column x, row y in generation t: 1 alive, 0 dead |
| `n` | number of live cells among the 8 neighbors (the Moore neighborhood) |
| B3/S23 | the rule in birth/survival notation: born with 3, survives with 2 or 3 |

The two lines encode the four classic rules:

- A live cell with fewer than 2 live neighbors dies (underpopulation).
- A live cell with 2 or 3 live neighbors lives on (survival).
- A live cell with more than 3 live neighbors dies (overpopulation).
- A dead cell with exactly 3 live neighbors becomes alive (reproduction).

## Intuition

The rules balance two pressures. Isolated cells starve and crowded cells suffocate, so live cells only persist in loose clusters. Birth needs exactly three parents, which is rare enough that growth stays controlled but common enough that patterns can rebuild themselves one step ahead, which is how a glider moves: every four generations it reappears one cell diagonally away, having been destroyed and recreated along the way.

All cells update **at the same time**. The next generation is computed entirely from the current one, so a cell born this step does not count as a neighbor until the next step. Updating cells one by one in place gives a different, wrong automaton.

The grid in the app wraps around: the right edge is glued to the left and the top to the bottom, making a torus. Gliders that leave one side come back on the other instead of crashing into a dead wall.

## Algorithm

1. Keep two grids: the current generation and the next.
2. For every cell, count the live cells among its 8 neighbors, wrapping the coordinates with modulo.
3. Apply the rules to decide the cell's state in the next grid.
4. Swap the grids and repeat.
5. For the fading trail, also keep an age per cell: generations alive, or generations since death, and color by it.

```
for y in 0..h:
    for x in 0..w:
        n = 0
        for dy in -1..1:
            for dx in -1..1:
                if (dx, dy) != (0, 0):
                    n += cur[(y + dy) mod h][(x + dx) mod w]
        next[y][x] = (n == 3) or (cur[y][x] and n == 2)
swap(cur, next)
```

With arrays (numpy), the neighbor count is the sum of the grid shifted (rolled) in the 8 directions.

## Complexity

| Quantity | Cost |
| --- | --- |
| one generation, W x H grid | O(W · H) time, 8 neighbor reads per cell |
| memory | O(W · H) for two grids (plus ages) |
| sparse patterns (active-cell lists) | O(live cells) per generation |
| Hashlife on repetitive patterns | can skip 2^k generations at once |

The plain double-buffered grid is simple and cache friendly. Hashlife (Bill Gosper, 1984) memoizes quadtree blocks and makes huge, regular patterns run astronomically far ahead.

## Parameters in the app

- **Start**: the initial pattern. *Random soup* fills 28% of cells at random; *Glider* and *Spaceship* move; *Pulsar* is a period-3 oscillator; *Glider gun* is Gosper's gun, firing a glider every 30 generations; *R-pentomino* and *Acorn* are tiny seeds that take over a thousand generations to settle.
- **Grid**: Small, Medium or Large (48, 96 or 160 columns).
- **Wrap edges**: on, the grid is a torus; off, cells outside the grid count as dead.
- **Speed**: generations per second.
- **Play / Pause**, **Step** (advance one generation), **Restart** (reload the start pattern), **Clear** (empty grid).
- Click or drag to draw live cells, right-drag to erase. Hover a cell to outline its neighborhood and see which rule decides its fate.
- Colors: newborn cells are bright and settle into cyan; cells that just died glow and cool to deep blue over 28 generations, leaving trails where life has been.

## Numerical notes and pitfalls

- **Update in place.** Writing the new state into the grid you are still reading corrupts the neighbor counts of later cells. Always compute into a separate buffer.
- **Edges.** Without wrapping, the dead border destroys gliders and changes the dynamics; with wrapping, a pattern can collide with its own exhaust. Neither equals the infinite plane, so large experiments use a big grid or a sparse representation.
- **Index wrapping.** In many languages `-1 % w` is negative. Use `(x + w - 1) % w` or a Euclidean remainder.
- **Off-by-one counting.** The cell itself is not one of its neighbors; summing the full 3 x 3 block and then subtracting the cell is a common shortcut and a common bug when forgotten.
- **Drawing input.** Fast mouse drags skip cells; fill the gap by stepping along the line between the previous and current cell.
- **Long transients.** "Nothing is happening" is hard to detect; compare against recent generations (or hash them) to find still lifes and oscillators.

## Applications

- Teaching emergence, self-organization and complexity.
- Models of growth and spread, such as forest fires, epidemics, crystal growth and traffic, built from related cellular automata.
- Procedural generation: cave and dungeon maps in games are made by running a few steps of a Life-like rule on random noise.
- Generative art and screen savers, often with fading trails like the app's.
- Theoretical computer science: Life is Turing complete and has been used to build logic gates, counters and even a working computer.
- Lattice-based simulation methods such as lattice gas and lattice Boltzmann fluid models share the same local-rule structure.

## History

John von Neumann studied self-reproducing cellular automata in the 1940s, using 29 states per cell. John Horton Conway, a mathematician at Cambridge, searched for a much simpler rule that was unpredictable yet not explosive and settled on B3/S23 in 1970. Martin Gardner published it in his "Mathematical Games" column in Scientific American in October 1970, and it became one of the first popular computer programs. Conway offered a prize for a pattern that grows forever; Bill Gosper's team at MIT won it the same year with the glider gun. The name "Life" reflects the birth, survival and death of the cells.

## References

- Martin Gardner, "The fantastic combinations of John Conway's new solitaire game 'life'", Scientific American, October 1970.
- Elwyn Berlekamp, John Conway and Richard Guy, *Winning Ways for Your Mathematical Plays*, volume 2, 1982.
- Nathaniel Johnston and Dave Greene, *Conway's Game of Life: Mathematics and Construction*, 2022.
- Stephen Wolfram, *A New Kind of Science*, 2002.
- Wikipedia, "Conway's Game of Life"; LifeWiki (conwaylife.com).

# Binary search tree

## Overview

A **binary search tree** (BST) stores keys in nodes so that every node obeys one rule: all keys in its left subtree are smaller, and all keys in its right subtree are larger. Looking for a key then means starting at the root and, at each node, going left or right; every comparison throws away a whole subtree. An in-order walk (left, node, right) visits the keys in sorted order.

How fast that is depends on the shape. Insert keys in a lucky order and the tree is bushy and short. Insert them already sorted and the tree is a chain. An **AVL tree** fixes this by repairing the shape after each insert and delete: it keeps the heights of the two subtrees of every node within 1 of each other, using small local **rotations** that never break the search rule.

The app draws the tree, replays each operation one comparison or rotation per frame, and glides nodes to their new places when a rotation moves them. The small number above each node is its balance factor.

## Formula

```math
left subtree < key < right subtree
h(node) = 1 + max(h(left), h(right))      a missing child has h = 0
bf(node) = h(left) - h(right)
AVL rule: |bf(node)| <= 1 for every node
```

| Symbol | Meaning |
| --- | --- |
| `h` | height: the number of nodes on the longest path down to a leaf (a leaf has `h = 1`) |
| `bf` | balance factor of a node, the height of its left subtree minus that of its right |
| `n` | the number of keys in the tree |

When an insert or delete leaves a node with `bf = 2` or `bf = -2`, one of four cases applies:

| Case | Shape | Fix |
| --- | --- | --- |
| Left-left | `bf = 2`, left child has `bf >= 0` | rotate the node right |
| Left-right | `bf = 2`, left child has `bf < 0` | rotate the left child left, then the node right |
| Right-right | `bf = -2`, right child has `bf <= 0` | rotate the node left |
| Right-left | `bf = -2`, right child has `bf > 0` | rotate the right child right, then the node left |

## Intuition

### Halving

At every node a search discards one side. If both sides hold about the same number of keys, the work halves each step, like binary search in a sorted array, but unlike an array a tree also inserts and deletes without shifting everything.

### A rotation is a local swap of parents

Rotating right at a node `x` with left child `y` lifts `y` into `x`'s place and makes `x` the right child of `y`. The subtree that sat between them (`y`'s right child) moves over to become `x`'s left child. The in-order sequence of keys does not change, so the search rule still holds, but the left side got one level shorter and the right side one level taller. The ringed node in the app is the one that moves down.

### Why a chain is bad

A tree of n keys has height at least about log2(n) but at most n. Height is what every operation pays, so keeping it near the minimum is the whole point of balancing.

## Algorithm

1. **Find:** from the root, stop if the key matches, go left if it is smaller, go right if it is larger. Reaching an empty child means the key is absent.
2. **Insert:** search as above and attach a new leaf where the search ended. A key already in the tree is ignored.
3. **Delete:** a leaf is removed. A node with one child is replaced by that child. A node with two children takes the value of its in-order successor (the smallest key in its right subtree), and that successor node is removed instead.
4. **Rebalance (AVL):** on the way back up from an insert or delete, recompute each node's height and balance factor. Where `|bf| = 2`, apply the matching rotation case from the table above.
5. **Traverse:** in-order is left, node, right; pre-order is node, left, right; post-order is left, right, node.

```
insert(node, key):
    if node is empty: return new leaf(key)
    if key < node.key: node.left = insert(node.left, key)
    else if key > node.key: node.right = insert(node.right, key)
    update height of node
    if balancing and |bf(node)| = 2: node = rotate to fix the case
    return node
```

## Complexity

| Operation | Balanced (AVL) | Unbalanced worst case |
| --- | --- | --- |
| find, insert, delete | O(log n) | O(n) |
| traversal | O(n) | O(n) |
| space | O(n) | O(n) |

Every operation costs O(h), where h is the height. For an AVL tree h < 1.44 log2(n + 2), so h is O(log n). A plain BST fed sorted keys becomes a chain with h = n. An insert needs at most one rotation case (one or two rotations); a delete may need one at every level on the way up, which is still O(log n).

## Parameters in the app

- **Key**: the value to act on, from -99 to 99.
- **Insert**, **Delete**, **Find**: run that operation on the key. Each starts a replay with a frame per comparison and per rotation. Find does not change the tree and ends with "found" or "not found".
- **Random 15**: clears the tree and inserts 15 distinct random keys between -99 and 99; a new click gives a new set.
- **Clear**: empties the tree.
- **Balance (AVL)**: switches AVL rotations on or off. Switching rebuilds the tree from the keys inserted so far (minus those deleted), in their original order, and shows the result at once. With it off, the balance factors are hidden.
- **In-order**, **Pre-order**, **Post-order**: visit every node in that order, one frame per node; the caption lists the keys visited so far.
- **Speed**, **Step**, **Restart**, **Step back**, **Play / Pause**, **Step**, **Finish**: transport controls. Space plays or pauses, Left and Right step one frame.
- **Drawing**: the node being compared or visited is filled with the trace color (the highlight), the node a rotation moves down is ringed in amber, and the number above a node is its balance factor, shown in the amber accent color when its absolute value is 2 or more, which appears in the frame just before a rotation fixes it. The header shows `n` (the number of keys) and the height, and the caption along the bottom says what the current frame is doing.

## Numerical notes and pitfalls

- **Sorted input degenerates.** Inserting 1, 2, 3, ... into a plain BST builds a chain, and every search becomes a linear scan. Turn balancing off in the app and insert ascending keys to see it.
- **Duplicates.** This tree keeps each key once and ignores a second insert. If you need duplicates, store a count in the node or decide a side (for example, equal goes right) and keep that rule everywhere.
- **Deleting a node with two children.** Copy the in-order successor's key into the node, then delete the successor, which has no left child. Forgetting this case, or removing the node without reconnecting its subtrees, loses keys.
- **Recursion depth.** Recursive code on an unbalanced tree can overflow the call stack at a height of tens of thousands. Balanced trees stay shallow; otherwise use an explicit stack.
- **Heights must be updated after rotating.** Rotate first, then recompute the lower node's height before the upper one, or the balance factors go stale.
- **Rotating the wrong way.** A left-right case needs two rotations. A single rotation of the node leaves it unbalanced on the other side.

## Applications

- Ordered maps and sets: sorted iteration, range queries, predecessor and successor. (Rust's `std::collections::BTreeMap` is a B-tree, a wider cousin that suits memory caches.)
- Database indexes, where B-trees and similar structures keep keys ordered on disk.
- Interval trees and other augmented trees that store extra data per node.
- The Linux scheduler, which keeps runnable tasks in a red-black tree.
- Priority-like structures that need both lookup by key and ordered traversal.

## History

The binary search tree was described independently around 1960 by P. F. Windley, A. D. Booth, A. J. T. Colin and T. N. Hibbard. Georgy Adelson-Velsky and Evgenii Landis published the first self-balancing version, now called the AVL tree, in 1962. Red-black trees, which relax the balance rule to make updates cheaper, were introduced by Leonidas Guibas and Robert Sedgewick in 1978.

## References

- T. H. Cormen, C. E. Leiserson, R. L. Rivest and C. Stein, *Introduction to Algorithms* (CLRS), chapters 12 and 13.
- D. E. Knuth, *The Art of Computer Programming*, volume 3, section 6.2.
- G. M. Adelson-Velsky and E. M. Landis, "An algorithm for the organization of information", Proceedings of the USSR Academy of Sciences, 1962.
- L. Guibas and R. Sedgewick, "A dichromatic framework for balanced trees", 1978.
- Wikipedia, "Binary search tree" and "AVL tree".

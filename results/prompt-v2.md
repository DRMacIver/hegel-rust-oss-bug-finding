# Subagent prompt v2 (weak-tier variant)

Identical to v1 except one added ground rule, testing whether a mechanical rule fixes haiku's generator over-constraining (batch-2 finding: written Discipline guidance alone didn't):

> - Mechanical generator rule: for EVERY bound you place on a generator (min_value/max_value/max_size/allow_nan(false)/etc.), you must either (a) cite in a comment the specific documentation line or contract that makes out-of-bound inputs invalid, or (b) keep the bound but add a companion unbounded no-panic test for the same operation. Bounds without one of these two justifications are forbidden.

First used: batch 3, petgraph-haiku.

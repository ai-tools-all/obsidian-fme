---
type: leetcode
id: 286
title: "Walls and Gates"
status: completed
pattern: multi-source-bfs
topics: [graph, bfs]
difficulty: medium
date: 2026-02-16 18:08:35
---

<!-- 2026-02-16 18:08:35 -->

# Problem 286: Walls and Gates (Islands and Treasure)

## Mistake
Multiple bugs: incorrect bounds check, wrong set method, unnecessary data structure.

## What Went Wrong

| Bug | ❌ Wrong | ✅ Correct |
|-----|---------|-----------|
| Bounds check | `0 < nx < m` | `0 <= nx < m` |
| Set method | `visited.append()` | `visited.add()` |
| Redundancy | Using `visited` set | Grid already tracks visited |

```python
# ❌ Buggy code
if 0 < nx < m and 0 < ny < n and grid[nx][ny] == inf_rep and (nx, ny) not in visited:
    visited.append((nx,ny))  # Sets don't have append()
```

## Why
1. `0 < nx` skips index 0, missing first row/column entirely
2. Sets use `add()` not `append()` - Python list method doesn't exist on sets
3. Modifying `grid[nx][ny]` from INF to distance already marks it visited

## Fix

```python
from collections import deque
class Solution:
    def islandsAndTreasure(self, grid: List[List[int]]) -> None:
        m, n = len(grid), len(grid[0])
        inf_rep = 2147483647

        queue = deque()
        
        for i in range(m):
            for j in range(n):
                if grid[i][j] == 0:
                    queue.append((i, j, 0))

        directions = [(0,1), (1,0), (-1,0), (0,-1)]
        
        while queue:
            x, y, dist = queue.popleft()

            for dx, dy in directions:
                nx, ny = x + dx, y + dy 

                if 0 <= nx < m and 0 <= ny < n and grid[nx][ny] == inf_rep:
                    grid[nx][ny] = dist + 1
                    queue.append((nx, ny, dist + 1))
```

## Key Lesson
- Bounds: Always use `0 <=` not `0 <` for inclusive lower bound
- Grid modification can serve as implicit visited tracker - no extra set needed
- Multi-source BFS: Start from all targets (0s), expand outward

---

## → solved by user ←

*Original buggy code provided. Corrected version above incorporates all fixes.*

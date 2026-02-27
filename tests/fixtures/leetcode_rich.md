---
type: leetcode
id: 133
title: "Clone Graph"
status: completed
pattern: graph-deep-copy-mapping
topics: [graph, dfs, bfs, hashmap, cloning]
difficulty: medium
date: 2026-02-22
---

# LC 133: Clone Graph – Deep Copy with Node Mapping

## Mistake

Core misconception: Thought you could simply copy the data structure without managing a mapping of original→clone references, leading to either infinite recursion or failure to reuse cloned nodes when revisiting them.

## What Went Wrong

### Iteration 1: No Mapping Structure
❌ **Wrong**
```python
def cloneGraph(node):
    if node is None:
        return None
    
    # Missing: where do we track cloned nodes?
    new_node = Node(node.val)
    
    # This loops forever when node has cycles (1 → 2 → 1)
    for nbr in node.neighbors:
        new_node.neighbors.append(cloneGraph(nbr))
```

**Problem:** When Node 1 → Node 2 → Node 1, the recursion never terminates. No way to say "I've already cloned Node 1, reuse that."

---

### Iteration 2: DFS with Map, But Missing Links
❌ **Wrong**
```python
def cloneGraph(node):
    cloned = {}
    
    def dfs(original_node):
        if original_node in cloned:
            return cloned[original_node]
        
        new_node = original_node.copy()  # Node has no .copy()!
        cloned[original_node] = new_node
        
        for nbr in original_node.neighbors:
            dfs(nbr)  # Returns cloned neighbor but discards it!
        
        # Missing: return statement
```

**Problems:**
- `Node` class has no `.copy()` method → must construct: `Node(original_node.val)`
- Loop calls `dfs(nbr)` but ignores return value → cloned neighbors never linked
- Missing `return` statement at end

---

### Iteration 3: BFS without Proper Clone-on-Discovery
❌ **Wrong**
```python
def cloneGraph(node):
    if node is None:
        return None 
    
    cloned = {} 
    queue = deque([node]) 
    cloned[node] = Node(node.val)  # Root cloned first ✓
    
    while queue:
        original_node = queue.popleft()
        
        # Check that root was cloned
        if original_node not in cloned:
            new_node = Node(original_node.val)
            cloned[original_node] = new_node    
        
        for nbr in original_node.neighbors:
            if nbr not in cloned:
                queue.append(nbr)  # Added to queue but NOT cloned yet!
        
        # Loop never links neighbors to the cloned node
```

**Problem:** When you discover uncloned neighbor, you add to queue but don't clone it. Next iteration tries to access `cloned[neighbor]` which doesn't exist yet.

---

## Why

### 1. Graph Cloning Requires Two Synchronous Operations
When you see an original node:
1. **Clone it** (create new Node, add to map)
2. **Add to work queue** (so you process its neighbors later)

These must happen **together**, not separately.

### 2. Traversal vs Reference Linking Are Different
- **Traversal:** "Have I visited this node?" (prevents infinite loops)
- **Linking:** "Where is the clone of this node?" (enables correct connections)

A simple `visited` set doesn't work because you need to **reuse the exact clone object**, not just mark as visited.

### 3. List Mutation During Iteration
When you build `cloned_node.neighbors` while looping, you're mutating the clone. The recursive/iterative process must capture results and connect them immediately.

---

## Fix

### DFS Approach ✅ **Correct**

```python
def cloneGraph(node):
    if node is None:
        return None
    
    cloned = {}
    
    def dfs(original_node):
        # Check if already cloned
        if original_node in cloned:
            return cloned[original_node]
        
        # Create clone, store immediately
        new_node = Node(original_node.val)
        cloned[original_node] = new_node
        
        # Recursively clone neighbors and link
        for nbr in original_node.neighbors:
            new_node.neighbors.append(dfs(nbr))
        
        return new_node
    
    return dfs(node)
```

**Key points:**
- Check if cloned **before** creating (avoid duplicates)
- Clone and map **immediately** on first encounter
- Capture return value of `dfs(nbr)` and append to neighbors
- Return the cloned node

---

### BFS Approach ✅ **Correct**

```python
from collections import deque

def cloneGraph(node):
    if node is None:
        return None
    
    cloned = {}
    queue = deque([node])
    
    # Clone root BEFORE entering loop
    cloned[node] = Node(node.val)
    
    while queue:
        original_node = queue.popleft()
        
        for nbr in original_node.neighbors:
            # If not yet cloned: create, map, queue
            if nbr not in cloned:
                cloned[nbr] = Node(nbr.val)
                queue.append(nbr)
            
            # Always link the neighbor (clone exists now)
            cloned[original_node].neighbors.append(cloned[nbr])
    
    return cloned[node]
```

**Key points:**
- Clone root **before loop** (special case)
- For each neighbor: if uncloned → clone AND queue together
- Link happens **after** cloning is guaranteed
- Process level-by-level, not recursive

---

## Key Lesson

### Pattern: Graph Deep Copy

**The core challenge:** Nodes reference each other (cycles). You can't recursively copy because:
- Naive recursion: infinite loop
- Solution: Memoization via map

**Template:**
```python
# DFS
cloned = {}
def clone(original):
    if original in cloned:
        return cloned[original]
    new = Node(original.val)
    cloned[original] = new
    new.neighbors = [clone(nbr) for nbr in original.neighbors]
    return new

# BFS
cloned = {node: Node(node.val)}
queue = deque([node])
while queue:
    curr = queue.popleft()
    for nbr in curr.neighbors:
        if nbr not in cloned:
            cloned[nbr] = Node(nbr.val)
            queue.append(nbr)
        cloned[curr].neighbors.append(cloned[nbr])
```

### Mental Model

**Before solving:** "I need to copy the graph structure" → static thinking

**After solving:** "I need to map original nodes to their clones, maintaining that mapping throughout traversal" → dynamic thinking

Key distinction:
- **Visited set:** "Have I processed this?" (boolean)
- **Clone map:** "What is the clone of this original?" (object reference)

Graph cloning problems always need the **map**, not just visited tracking.

---

## Complexity

- **DFS:**
  - Time: O(V + E) where V = nodes, E = edges (each node/edge processed once)
  - Space: O(V) for the map + O(V) for recursion stack = O(V)

- **BFS:**
  - Time: O(V + E) (same)
  - Space: O(V) for the map + O(V) for queue = O(V)

Both equivalent; DFS simpler code, BFS iterative (no stack overflow risk on large graphs).

---

## → solved by user (both approaches) ←

**DFS Final:**
```python
def cloneGraph(node):
    if node is None:
        return None
    
    cloned = {}
    
    def dfs(original_node):
        if original_node in cloned:
            return cloned[original_node]
        
        new_node = Node(original_node.val)
        cloned[original_node] = new_node
        
        for nbr in original_node.neighbors:
            new_node.neighbors.append(dfs(nbr))
        
        return new_node
    
    return dfs(node)
```

**BFS Final:**
```python
from collections import deque

def cloneGraph(node):
    if node is None:
        return None
    
    cloned = {}
    queue = deque([node])
    cloned[node] = Node(node.val)
    
    while queue:
        original_node = queue.popleft()
        
        for nbr in original_node.neighbors:
            if nbr not in cloned:
                cloned[nbr] = Node(nbr.val)
                queue.append(nbr)
            
            cloned[original_node].neighbors.append(cloned[nbr])
    
    return cloned[node]
```

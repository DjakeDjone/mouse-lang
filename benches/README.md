# Query Performance Benchmarks

This directory contains Criterion benchmarks for testing the performance of the query engine.

## Running Benchmarks

### Run all benchmarks
```bash
cargo bench
```

### Run a specific benchmark
```bash
cargo bench query_simple_equals
```

### Run benchmarks with specific sample size
```bash
cargo bench -- --sample-size 50
```

### Benchmark Results

#### Indexed vs Non-Indexed Performance (100,000 rows)

| Query Type | No Index | With Index | Speedup | Performance |
|------------|----------|------------|---------|-------------|
| **Simple Equals** | 75.35 ms | 17.46 ms | **4.3x faster** | ✅ Significant improvement |
| **OR Multiple Conditions** | 92.59 ms | 90.70 ms | 1.02x faster | ⚠️ Minimal improvement |
| **AND Conditions (Range)** | 74.70 ms | 74.66 ms | ~1.0x | ❌ No improvement |
| **Timestamp Range** | 74.35 ms | 74.24 ms | ~1.0x | ❌ No improvement |
| **Complex Nested** | 86.80 ms | 88.54 ms | **0.98x** | ❌ Actually slower |

#### Summary

**Index Performance Analysis:**

**✅ What Works:**
- **Simple equality queries** see massive **4.3x speedup** (75ms → 17ms)
- Indexes are clearly effective for `Equals` operations on indexed columns

**⚠️ What Needs Improvement:**
1. **Range queries** (GreaterThan/LessThan with AND) show **zero improvement**
   - Suggests indexes aren't being utilized for range operations
   - Both `query_and_conditions` and `query_timestamp_range` demonstrate this pattern
   - Consider implementing B-tree indexes for better range query support

2. **OR conditions** show **minimal improvement** (~2%)
   - Multiple index lookups with union operations may have high overhead
   - Current index architecture may not be optimal for OR queries

3. **Complex nested queries** are **actually slower with indexes** (2% regression)
   - Index lookup overhead outweighs benefits for complex query patterns
   - Query planner should consider skipping indexes for these cases

**Recommendations:**
1. Implement B-tree or sorted indexes to support efficient range queries
2. Add query planning logic to decide when to use indexes based on query type
3. Consider index-only scans for simple equality queries
4. Profile OR query execution to understand overhead sources

**Current Best Practice:**
- **Use indexes for:** Simple `Equals` queries ✅
- **Skip indexes for:** Range queries, complex nested queries (currently no benefit)

**Test Configuration:**
- Dataset: 100,000 rows with 5 columns (id, column1, column2, date, amount)
- Runtime: Tokio async runtime
- Backend: Plotters (Gnuplot not found)

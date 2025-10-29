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

#### Latest Results (100,000 rows)

| Benchmark | Mean Time | Range | Notes |
|-----------|-----------|-------|-------|
| query_simple_equals | 112.71 ms | 111.17 - 114.23 ms | Simple equality query on indexed column |
| query_or_multiple_conditions | 128.84 ms | 127.60 - 130.16 ms | OR conditions across multiple columns |
| query_and_conditions | 97.44 ms | 96.06 - 98.79 ms | Range query with AND conditions |
| query_timestamp_range | 101.24 ms | 100.48 - 102.01 ms | Timestamp range filtering |
| query_complex_nested | 114.99 ms | 113.52 - 116.46 ms | Nested AND/OR with multiple conditions |

#### Summary

The query engine demonstrates consistent performance across different query types on a dataset of 100,000 rows:

- **Fastest Query**: AND conditions (range queries) at ~97ms
- **Slowest Query**: OR with multiple conditions at ~129ms
- **Average Query Time**: ~111ms across all benchmark types

**Key Observations:**
1. AND conditions (range queries) are the most efficient
2. Complex nested queries perform similarly to simple queries
3. All queries complete in under 131ms, providing sub-second response times
4. Some outliers detected in OR and timestamp queries suggest potential for further optimization

*Note: The ~90% improvement shown by Criterion for some queries is from comparing against a previous run with a different dataset size, not from actual optimizations.*

**Test Configuration:**
- Dataset: 100,000 rows with 5 columns (id, column1, column2, date, amount)
- Runtime: Tokio async runtime
- Backend: Plotters (Gnuplot not found)

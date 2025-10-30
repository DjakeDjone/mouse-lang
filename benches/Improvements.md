# MouseDB Performance Improvements

+ Encoding: bincode instead of JSON (+25% Performance)
+ Partitioning: RocksDB as Chore instead of own kv engine: (-7.03 - -13% Performance for Queries): likely because the query engine is not optimized for RocksDB, but get_by_id is way faster, + better scaling + necessary for Indexed Queries

## Not Yet Implemented Improvements

+ Compression: LZ4 or Zstd
+ Indexing: B-tree or Hashmap
+ In-Memory Caching

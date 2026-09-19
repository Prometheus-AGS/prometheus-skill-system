Qdrant sizing guidance states that a cluster should be planned for roughly 1.5 times the raw vector footprint in RAM when the HNSW index is held fully in memory. [src:a1b2c3d4]

Snapshot restore requires additional headroom during the copy, which is a distinct capacity concern from steady-state indexing. [src:a1b2c3d4]

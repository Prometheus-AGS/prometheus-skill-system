## Why

The installed SurrealDB 3.2.4 service reserved a 33,285,996,544-byte RocksDB
block cache on a shared workstation. During learning-queue recovery, swap grew
to 36.6 GB and durable operation queries stalled for minutes.

## What Changes

- Set the documented `SURREAL_ROCKSDB_BLOCK_CACHE_SIZE` environment variable to
  1 GiB in both managed native service templates.
- Include the SurrealDB definition in render-only output and start a reloaded
  RunAtLoad launch agent exactly once.
- Regenerate the service manifest and redeploy the managed services.
- Verify the deployed SurrealDB log reports the bounded cache and the durable
  learning queue resumes.

## Capabilities

### New Capabilities

- `surrealdb-native-resource-bounds`: Keeps the owned local database's RocksDB
  block cache within an explicit budget while agent tools share the host.

### Modified Capabilities

None.

## Impact

The block cache may perform more disk reads than the dynamic half-memory
default. Other SurrealDB allocations remain outside this limit, but bounding
the cache removes the observed 33.3 GB reservation that increased host memory
pressure. Removing the redundant kick relies on launchd's documented
RunAtLoad bootstrap behavior already declared by each affected job.

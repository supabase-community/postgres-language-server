select type, name, total_bytes > 0, total_nblocks, free_bytes > 0, free_chunks
from pg_backend_memory_contexts where name = 'Caller tuples';

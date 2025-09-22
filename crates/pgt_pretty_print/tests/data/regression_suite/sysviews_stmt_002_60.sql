select type, name, ident, level, total_bytes >= free_bytes
  from pg_backend_memory_contexts where level = 1;

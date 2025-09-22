CREATE DOMAIN domain4 AS text[]
  DEFAULT ('{"This", "is", "' || foo(4) || '","the", "real", "world"}')::TEXT[];

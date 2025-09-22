SELECT oid, opcname FROM pg_opclass WHERE NOT amvalidate(oid);

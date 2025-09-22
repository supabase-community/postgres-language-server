SELECT a.amname FROM pg_class c, pg_am a
  WHERE c.relname = 'am_partitioned' AND a.oid = c.relam;

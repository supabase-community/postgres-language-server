SELECT a.amname FROM pg_class c, pg_am a
  WHERE c.relname = 'tableam_parted_heap2' AND a.oid = c.relam;

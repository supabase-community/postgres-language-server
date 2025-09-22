UPDATE foo_parted_v SET a = 1, c = c || '->P1' WHERE a = 2 AND c = 'P2'
  RETURNING 'P2:'||old.dummy, 'P1:'||new.dummy;

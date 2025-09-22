SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'temporal_fk_mltrng2mltrng_fk';

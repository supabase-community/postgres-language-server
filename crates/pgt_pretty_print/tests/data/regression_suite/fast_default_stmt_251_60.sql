SELECT count(*)
  FROM pg_attribute
  WHERE attrelid = 'ft1'::regclass AND
    (attmissingval IS NOT NULL OR atthasmissing);

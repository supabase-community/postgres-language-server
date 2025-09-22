CREATE FUNCTION leak2(integer,integer) RETURNS boolean
  AS $$begin raise notice 'leak % %', $1, $2; return $1 > $2; end$$
  LANGUAGE plpgsql immutable;

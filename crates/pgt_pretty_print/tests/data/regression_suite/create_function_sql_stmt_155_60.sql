CREATE FUNCTION part_hashint4_error(value int4, seed int8) RETURNS int8
LANGUAGE SQL STRICT IMMUTABLE PARALLEL SAFE AS
$$ SELECT value + seed + random()::int/0 $$;

CREATE FUNCTION double_append(anyarray, anyelement) RETURNS SETOF anyarray
LANGUAGE SQL IMMUTABLE AS
$$ SELECT array_append($1, $2) || array_append($1, $2) $$;

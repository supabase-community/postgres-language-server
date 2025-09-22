CREATE FUNCTION bar_int4_text(int4) RETURNS text LANGUAGE SQL AS
$$ SELECT ('bar'::text || $1::text); $$;

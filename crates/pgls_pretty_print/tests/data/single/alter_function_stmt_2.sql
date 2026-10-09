ALTER FUNCTION my_schema.compute_totals(integer, text) IMMUTABLE LEAKPROOF STRICT SECURITY DEFINER PARALLEL SAFE COST 10 ROWS 5 SET search_path = public, pg_temp RESET work_mem;

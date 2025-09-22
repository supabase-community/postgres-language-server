CREATE FUNCTION mv_action() RETURNS bool LANGUAGE sql AS
	'DECLARE c CURSOR WITH HOLD FOR SELECT public.unwanted_grant(); SELECT true';

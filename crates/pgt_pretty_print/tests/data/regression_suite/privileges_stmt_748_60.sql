CREATE FUNCTION sro_trojan() RETURNS trigger LANGUAGE plpgsql AS
	'BEGIN PERFORM public.unwanted_grant(); RETURN NULL; END';

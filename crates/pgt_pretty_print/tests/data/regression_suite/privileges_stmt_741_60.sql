CREATE FUNCTION unwanted_grant() RETURNS void LANGUAGE sql AS
	'GRANT regress_priv_group2 TO regress_sro_user';

CREATE FUNCTION sro_ifun(int) RETURNS int AS $$
BEGIN
	-- Below we set the table's owner to regress_sro_user
	ASSERT current_user = 'regress_sro_user',
		format('sro_ifun(%s) called by %s', $1, current_user);
	RETURN $1;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

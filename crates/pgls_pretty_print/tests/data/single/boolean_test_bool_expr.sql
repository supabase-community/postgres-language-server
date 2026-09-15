SELECT id
	FROM s.units
	WHERE (units.has_history OR units.has_shares OR units.has_calls) IS TRUE
	  AND (units.is_active AND units.is_visible) IS NOT FALSE;

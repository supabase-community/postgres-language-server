SELECT * FROM my_property_normal v
		WHERE f_leak('passwd') AND f_leak(passwd);

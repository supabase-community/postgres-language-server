SELECT * FROM my_property_secure v
		WHERE f_leak('passwd') AND f_leak(passwd);

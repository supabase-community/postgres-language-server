PREPARE p2 AS SELECT * FROM my_property_secure WHERE f_leak(passwd);

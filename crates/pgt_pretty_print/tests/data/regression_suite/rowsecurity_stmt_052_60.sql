SELECT * FROM document NATURAL JOIN category WHERE f_leak(dtitle) ORDER BY did;

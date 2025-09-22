CREATE VIEW mysecview10 WITH (security_invoker=100)	-- Error
       AS SELECT * FROM tbl1 WHERE a <> 100;

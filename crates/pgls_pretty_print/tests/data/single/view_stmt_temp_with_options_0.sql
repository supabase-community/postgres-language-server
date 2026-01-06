CREATE OR REPLACE TEMP VIEW view_with_opts ("ID", nickname)
WITH (security_barrier)
AS SELECT id, nickname FROM accounts
WITH LOCAL CHECK OPTION;

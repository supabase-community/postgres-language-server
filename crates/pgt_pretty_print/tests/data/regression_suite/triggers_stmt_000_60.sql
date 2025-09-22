CREATE FUNCTION trigger_return_old ()
        RETURNS trigger
        AS 'regresslib'
        LANGUAGE C;

select * from pg_proc where proname ~ '^abcd(x|(?=\w\w)q)';

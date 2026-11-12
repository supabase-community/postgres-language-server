explain (analyze, verbose false, format json) select * from users;
explain (costs off) update users set name = 'x' returning with (old as o) o.name;
analyze (verbose, skip_locked true) users;
copy users from stdin with (format csv, header true, delimiter ',');
copy users to stdout (format csv, force_quote *);
vacuum (verbose, analyze true, index_cleanup off, parallel 2) only users, posts (id);
vacuum full verbose users;

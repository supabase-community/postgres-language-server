select count(*) = 1 as dot_found
  from pg_ls_dir('.', false, false) as ls where ls = '.';

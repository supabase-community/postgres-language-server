create function set_and_report_role() returns text as
  $$ select current_setting('role') $$ language sql parallel safe
  set role = regress_parallel_worker;

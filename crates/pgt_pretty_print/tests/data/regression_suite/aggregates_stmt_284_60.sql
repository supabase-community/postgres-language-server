select string_agg(a,',') from (values('aaaa'),('bbbb'),('cccc')) g(a);

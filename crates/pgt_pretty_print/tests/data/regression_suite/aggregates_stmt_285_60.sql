select string_agg(a,',') from (values('aaaa'),(null),('bbbb'),('cccc')) g(a);

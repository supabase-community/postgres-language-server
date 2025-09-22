select 'foo'::text in (select 'bar'::name union all select 'bar'::name);

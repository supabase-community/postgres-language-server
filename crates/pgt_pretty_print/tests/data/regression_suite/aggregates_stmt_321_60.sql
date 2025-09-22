select array_dims(array_agg(s)) from (select * from pagg_test) s;

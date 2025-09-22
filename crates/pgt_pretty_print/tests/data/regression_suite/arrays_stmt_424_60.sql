select array(select array['Hello', i::text] from generate_series(9,11) i);

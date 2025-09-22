select * from test_tsquery, to_tsquery('new') q where txtsample @@ q;

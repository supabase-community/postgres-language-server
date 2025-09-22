select * from test_tsquery, to_tsquery('english', 'new') q where txtsample @@ q;

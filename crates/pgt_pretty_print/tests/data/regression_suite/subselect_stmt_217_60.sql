select x, x from
    (select (select random()) as x from (values(1),(2)) v(y)) ss;

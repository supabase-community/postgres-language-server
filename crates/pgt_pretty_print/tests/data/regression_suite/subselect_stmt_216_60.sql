select x, x from
    (select (select now()) as x from (values(1),(2)) v(y)) ss;

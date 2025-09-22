create temp view shipped_view as
    select * from shipped where ttype = 'wt';

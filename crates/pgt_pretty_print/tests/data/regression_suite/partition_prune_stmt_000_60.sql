create function explain_analyze(query text) returns setof text
language plpgsql as
$$
declare
    ln text;
begin
    for ln in
        execute format('explain (analyze, costs off, summary off, timing off, buffers off) %s',
            query)
    loop
        ln := regexp_replace(ln, 'Maximum Storage: \d+', 'Maximum Storage: N');
        return next ln;
    end loop;
end;
$$;

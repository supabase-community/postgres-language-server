SELECT
    -- fixed-width by-value datum
    (array_agg(id ORDER BY id DESC NULLS FIRST))[0:5],
    -- fixed-width by-ref datum
    (array_agg(abort_increasing ORDER BY abort_increasing DESC NULLS LAST))[0:5],
    -- variable-width datum
    (array_agg(id::text ORDER BY id::text DESC NULLS LAST))[0:5],
    -- fixed width by-value datum tuplesort
    percentile_disc(0.99) WITHIN GROUP (ORDER BY id),
    -- ensure state is shared
    percentile_disc(0.01) WITHIN GROUP (ORDER BY id),
    -- fixed width by-ref datum tuplesort
    percentile_disc(0.8) WITHIN GROUP (ORDER BY abort_increasing),
    -- variable width by-ref datum tuplesort
    percentile_disc(0.2) WITHIN GROUP (ORDER BY id::text),
    -- multi-column tuplesort
    rank('00000000-0000-0000-0000-000000000000', '2', '2') WITHIN GROUP (ORDER BY noabort_increasing, id, id::text)
FROM (
    SELECT * FROM abbrev_abort_uuids
    UNION ALL
    SELECT NULL, NULL, NULL, NULL, NULL) s;

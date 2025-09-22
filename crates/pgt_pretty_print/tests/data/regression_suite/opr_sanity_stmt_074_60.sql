SELECT pp.oid::regprocedure as proc, pp.provolatile as vp, pp.proleakproof as lp,
       po.oid::regprocedure as opr, po.provolatile as vo, po.proleakproof as lo
FROM pg_proc pp, pg_proc po, pg_operator o, pg_amproc ap, pg_amop ao
WHERE pp.oid = ap.amproc AND po.oid = o.oprcode AND o.oid = ao.amopopr AND
    ao.amopmethod = (SELECT oid FROM pg_am WHERE amname = 'btree') AND
    ao.amopfamily = ap.amprocfamily AND
    ao.amoplefttype = ap.amproclefttype AND
    ao.amoprighttype = ap.amprocrighttype AND
    ap.amprocnum = 1 AND
    (pp.provolatile != po.provolatile OR
     pp.proleakproof != po.proleakproof)
ORDER BY 1;

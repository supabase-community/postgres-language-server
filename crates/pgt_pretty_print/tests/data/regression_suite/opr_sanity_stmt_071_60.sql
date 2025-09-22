WITH funcdescs AS (
  SELECT p.oid as p_oid, proname, o.oid as o_oid,
    pd.description as prodesc,
    'implementation of ' || oprname || ' operator' as expecteddesc,
    od.description as oprdesc
  FROM pg_proc p JOIN pg_operator o ON oprcode = p.oid
       LEFT JOIN pg_description pd ON
         (pd.objoid = p.oid and pd.classoid = p.tableoid and pd.objsubid = 0)
       LEFT JOIN pg_description od ON
         (od.objoid = o.oid and od.classoid = o.tableoid and od.objsubid = 0)
  WHERE o.oid <= 9999
)
SELECT p_oid, proname, prodesc FROM funcdescs
  WHERE prodesc IS DISTINCT FROM expecteddesc
    AND oprdesc NOT LIKE 'deprecated%'
ORDER BY 1;

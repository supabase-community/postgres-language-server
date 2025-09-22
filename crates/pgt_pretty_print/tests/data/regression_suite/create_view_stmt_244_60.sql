delete from pg_depend where
  objid = (select oid from pg_rewrite
           where ev_class = 'tt14v'::regclass and rulename = '_RETURN')
  and refobjsubid = 4
returning pg_describe_object(classid, objid, objsubid) as obj,
          pg_describe_object(refclassid, refobjid, refobjsubid) as ref,
          deptype;

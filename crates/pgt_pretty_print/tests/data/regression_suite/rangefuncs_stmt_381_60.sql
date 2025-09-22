delete from pg_depend where
  objid = (select oid from pg_rewrite
           where ev_class = 'usersview'::regclass and rulename = '_RETURN')
  and refobjsubid = 5
returning pg_describe_object(classid, objid, objsubid) as obj,
          pg_describe_object(refclassid, refobjid, refobjsubid) as ref,
          deptype;

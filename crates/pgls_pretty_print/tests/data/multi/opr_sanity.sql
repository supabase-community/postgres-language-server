SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE p1.prolang = 0 OR p1.prorettype = 0 OR
       p1.pronargs < 0 OR
       p1.pronargdefaults < 0 OR
       p1.pronargdefaults > p1.pronargs OR
       array_lower(p1.proargtypes, 1) != 0 OR
       array_upper(p1.proargtypes, 1) != p1.pronargs-1 OR
       0::oid = ANY (p1.proargtypes) OR
       procost <= 0 OR
       CASE WHEN proretset THEN prorows <= 0 ELSE prorows != 0 END OR
       prokind NOT IN ('f', 'a', 'w', 'p') OR
       provolatile NOT IN ('i', 's', 'v') OR
       proparallel NOT IN ('s', 'r', 'u');

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE prosrc IS NULL;

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE (prosrc = '' OR prosrc = '-') AND prosqlbody IS NULL;

SELECT p1.oid, p1.proname
FROM pg_proc AS p1
WHERE proretset AND prokind != 'f';

SELECT p1.oid, p1.proname
FROM pg_proc AS p1
WHERE prosecdef
ORDER BY 1;

SELECT p1.oid, p1.proname
FROM pg_proc AS p1
WHERE (pronargdefaults <> 0) != (proargdefaults IS NOT NULL);

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE prolang = 13 AND (probin IS NULL OR probin = '' OR probin = '-');

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE prolang != 13 AND probin IS NOT NULL;

SELECT p1.oid, p1.proname, p2.oid, p2.proname
FROM pg_proc AS p1, pg_proc AS p2
WHERE p1.oid != p2.oid AND
    p1.proname = p2.proname AND
    p1.pronargs = p2.pronargs AND
    p1.proargtypes = p2.proargtypes;

SELECT p1.oid, p1.proname, p2.oid, p2.proname
FROM pg_proc AS p1, pg_proc AS p2
WHERE p1.oid < p2.oid AND
    p1.prosrc = p2.prosrc AND
    p1.prolang = 12 AND p2.prolang = 12 AND
    (p1.prokind != 'a' OR p2.prokind != 'a') AND
    (p1.prolang != p2.prolang OR
     p1.prokind != p2.prokind OR
     p1.prosecdef != p2.prosecdef OR
     p1.proleakproof != p2.proleakproof OR
     p1.proisstrict != p2.proisstrict OR
     p1.proretset != p2.proretset OR
     p1.provolatile != p2.provolatile OR
     p1.pronargs != p2.pronargs);

SELECT DISTINCT p1.prorettype::regtype, p2.prorettype::regtype
FROM pg_proc AS p1, pg_proc AS p2
WHERE p1.oid != p2.oid AND
    p1.prosrc = p2.prosrc AND
    p1.prolang = 12 AND p2.prolang = 12 AND
    p1.prokind != 'a' AND p2.prokind != 'a' AND
    p1.prosrc NOT LIKE E'range\\_constructor_' AND
    p2.prosrc NOT LIKE E'range\\_constructor_' AND
    p1.prosrc NOT LIKE E'multirange\\_constructor_' AND
    p2.prosrc NOT LIKE E'multirange\\_constructor_' AND
    (p1.prorettype < p2.prorettype)
ORDER BY 1, 2;

SELECT DISTINCT p1.proargtypes[0]::regtype, p2.proargtypes[0]::regtype
FROM pg_proc AS p1, pg_proc AS p2
WHERE p1.oid != p2.oid AND
    p1.prosrc = p2.prosrc AND
    p1.prolang = 12 AND p2.prolang = 12 AND
    p1.prokind != 'a' AND p2.prokind != 'a' AND
    p1.prosrc NOT LIKE E'range\\_constructor_' AND
    p2.prosrc NOT LIKE E'range\\_constructor_' AND
    p1.prosrc NOT LIKE E'multirange\\_constructor_' AND
    p2.prosrc NOT LIKE E'multirange\\_constructor_' AND
    (p1.proargtypes[0] < p2.proargtypes[0])
ORDER BY 1, 2;

SELECT DISTINCT p1.proargtypes[1]::regtype, p2.proargtypes[1]::regtype
FROM pg_proc AS p1, pg_proc AS p2
WHERE p1.oid != p2.oid AND
    p1.prosrc = p2.prosrc AND
    p1.prolang = 12 AND p2.prolang = 12 AND
    p1.prokind != 'a' AND p2.prokind != 'a' AND
    p1.prosrc NOT LIKE E'range\\_constructor_' AND
    p2.prosrc NOT LIKE E'range\\_constructor_' AND
    p1.prosrc NOT LIKE E'multirange\\_constructor_' AND
    p2.prosrc NOT LIKE E'multirange\\_constructor_' AND
    (p1.proargtypes[1] < p2.proargtypes[1])
ORDER BY 1, 2;

SELECT DISTINCT p1.proargtypes[2]::regtype, p2.proargtypes[2]::regtype
FROM pg_proc AS p1, pg_proc AS p2
WHERE p1.oid != p2.oid AND
    p1.prosrc = p2.prosrc AND
    p1.prolang = 12 AND p2.prolang = 12 AND
    p1.prokind != 'a' AND p2.prokind != 'a' AND
    (p1.proargtypes[2] < p2.proargtypes[2])
ORDER BY 1, 2;

SELECT DISTINCT p1.proargtypes[3]::regtype, p2.proargtypes[3]::regtype
FROM pg_proc AS p1, pg_proc AS p2
WHERE p1.oid != p2.oid AND
    p1.prosrc = p2.prosrc AND
    p1.prolang = 12 AND p2.prolang = 12 AND
    p1.prokind != 'a' AND p2.prokind != 'a' AND
    (p1.proargtypes[3] < p2.proargtypes[3])
ORDER BY 1, 2;

SELECT DISTINCT p1.proargtypes[4]::regtype, p2.proargtypes[4]::regtype
FROM pg_proc AS p1, pg_proc AS p2
WHERE p1.oid != p2.oid AND
    p1.prosrc = p2.prosrc AND
    p1.prolang = 12 AND p2.prolang = 12 AND
    p1.prokind != 'a' AND p2.prokind != 'a' AND
    (p1.proargtypes[4] < p2.proargtypes[4])
ORDER BY 1, 2;

SELECT DISTINCT p1.proargtypes[5]::regtype, p2.proargtypes[5]::regtype
FROM pg_proc AS p1, pg_proc AS p2
WHERE p1.oid != p2.oid AND
    p1.prosrc = p2.prosrc AND
    p1.prolang = 12 AND p2.prolang = 12 AND
    p1.prokind != 'a' AND p2.prokind != 'a' AND
    (p1.proargtypes[5] < p2.proargtypes[5])
ORDER BY 1, 2;

SELECT DISTINCT p1.proargtypes[6]::regtype, p2.proargtypes[6]::regtype
FROM pg_proc AS p1, pg_proc AS p2
WHERE p1.oid != p2.oid AND
    p1.prosrc = p2.prosrc AND
    p1.prolang = 12 AND p2.prolang = 12 AND
    p1.prokind != 'a' AND p2.prokind != 'a' AND
    (p1.proargtypes[6] < p2.proargtypes[6])
ORDER BY 1, 2;

SELECT DISTINCT p1.proargtypes[7]::regtype, p2.proargtypes[7]::regtype
FROM pg_proc AS p1, pg_proc AS p2
WHERE p1.oid != p2.oid AND
    p1.prosrc = p2.prosrc AND
    p1.prolang = 12 AND p2.prolang = 12 AND
    p1.prokind != 'a' AND p2.prokind != 'a' AND
    (p1.proargtypes[7] < p2.proargtypes[7])
ORDER BY 1, 2;

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE p1.prorettype = 'internal'::regtype AND NOT
    'internal'::regtype = ANY (p1.proargtypes);

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE p1.prorettype IN
    ('anyelement'::regtype, 'anyarray'::regtype, 'anynonarray'::regtype,
     'anyenum'::regtype)
  AND NOT
    ('anyelement'::regtype = ANY (p1.proargtypes) OR
     'anyarray'::regtype = ANY (p1.proargtypes) OR
     'anynonarray'::regtype = ANY (p1.proargtypes) OR
     'anyenum'::regtype = ANY (p1.proargtypes) OR
     'anyrange'::regtype = ANY (p1.proargtypes) OR
     'anymultirange'::regtype = ANY (p1.proargtypes))
ORDER BY 2;

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE p1.prorettype IN ('anyrange'::regtype, 'anymultirange'::regtype)
  AND NOT
    ('anyrange'::regtype = ANY (p1.proargtypes) OR
      'anymultirange'::regtype = ANY (p1.proargtypes))
ORDER BY 2;

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE p1.prorettype IN
    ('anycompatible'::regtype, 'anycompatiblearray'::regtype,
     'anycompatiblenonarray'::regtype)
  AND NOT
    ('anycompatible'::regtype = ANY (p1.proargtypes) OR
     'anycompatiblearray'::regtype = ANY (p1.proargtypes) OR
     'anycompatiblenonarray'::regtype = ANY (p1.proargtypes) OR
     'anycompatiblerange'::regtype = ANY (p1.proargtypes))
ORDER BY 2;

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE p1.prorettype = 'anycompatiblerange'::regtype
  AND NOT
     'anycompatiblerange'::regtype = ANY (p1.proargtypes)
ORDER BY 2;

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE 'cstring'::regtype = ANY (p1.proargtypes)
    AND NOT EXISTS(SELECT 1 FROM pg_type WHERE typinput = p1.oid)
    AND NOT EXISTS(SELECT 1 FROM pg_conversion WHERE conproc = p1.oid)
    AND p1.oid != 'shell_in(cstring)'::regprocedure
ORDER BY 1;

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE  p1.prorettype = 'cstring'::regtype
    AND NOT EXISTS(SELECT 1 FROM pg_type WHERE typoutput = p1.oid)
    AND NOT EXISTS(SELECT 1 FROM pg_type WHERE typmodout = p1.oid)
    AND p1.oid != 'shell_out(void)'::regprocedure
ORDER BY 1;

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE proallargtypes IS NOT NULL AND
    array_length(proallargtypes,1) < array_length(proargtypes,1);

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE proargmodes IS NOT NULL AND
    array_length(proargmodes,1) < array_length(proargtypes,1);

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE proargnames IS NOT NULL AND
    array_length(proargnames,1) < array_length(proargtypes,1);

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE proallargtypes IS NOT NULL AND proargmodes IS NOT NULL AND
    array_length(proallargtypes,1) <> array_length(proargmodes,1);

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE proallargtypes IS NOT NULL AND proargnames IS NOT NULL AND
    array_length(proallargtypes,1) <> array_length(proargnames,1);

SELECT p1.oid, p1.proname
FROM pg_proc as p1
WHERE proargmodes IS NOT NULL AND proargnames IS NOT NULL AND
    array_length(proargmodes,1) <> array_length(proargnames,1);

SELECT p1.oid, p1.proname, p1.proargtypes, p1.proallargtypes, p1.proargmodes
FROM pg_proc as p1
WHERE proallargtypes IS NOT NULL AND
  ARRAY(SELECT unnest(proargtypes)) <>
  ARRAY(SELECT proallargtypes[i]
        FROM generate_series(1, array_length(proallargtypes, 1)) g(i)
        WHERE proargmodes IS NULL OR proargmodes[i] IN ('i', 'b', 'v'));

SELECT oid::regprocedure, provariadic::regtype, proargtypes::regtype[]
FROM pg_proc
WHERE provariadic != 0
AND case proargtypes[array_length(proargtypes, 1)-1]
	WHEN '"any"'::regtype THEN '"any"'::regtype
	WHEN 'anyarray'::regtype THEN 'anyelement'::regtype
	WHEN 'anycompatiblearray'::regtype THEN 'anycompatible'::regtype
	ELSE (SELECT t.oid
		  FROM pg_type t
		  WHERE t.typarray = proargtypes[array_length(proargtypes, 1)-1])
	END  != provariadic;

SELECT oid::regprocedure, proargmodes, provariadic
FROM pg_proc
WHERE (proargmodes IS NOT NULL AND 'v' = any(proargmodes))
    IS DISTINCT FROM
    (provariadic != 0);

SELECT p1.oid, p1.proname, p2.oid, p2.proname
FROM pg_proc AS p1, pg_proc AS p2
WHERE p2.oid = p1.prosupport AND
    (p2.prorettype != 'internal'::regtype OR p2.proretset OR p2.pronargs != 1
     OR p2.proargtypes[0] != 'internal'::regtype);

SELECT p1.oid, p1.proname
FROM pg_proc as p1 LEFT JOIN pg_description as d
     ON p1.tableoid = d.classoid and p1.oid = d.objoid and d.objsubid = 0
WHERE d.classoid IS NULL AND p1.oid <= 9999;

SELECT p1.oid::regprocedure
FROM pg_proc p1 JOIN pg_namespace pn
     ON pronamespace = pn.oid
WHERE nspname = 'pg_catalog' AND proleakproof
ORDER BY 1;

SELECT p1.oid::regprocedure
FROM pg_proc p1 JOIN pg_namespace pn
     ON pronamespace = pn.oid
WHERE nspname = 'pg_catalog' AND proleakproof AND pronargs = 0
ORDER BY 1;

select proname, oid from pg_catalog.pg_proc
where proname in (
  'lo_open',
  'lo_close',
  'lo_creat',
  'lo_create',
  'lo_unlink',
  'lo_lseek',
  'lo_lseek64',
  'lo_tell',
  'lo_tell64',
  'lo_truncate',
  'lo_truncate64',
  'loread',
  'lowrite')
and pronamespace = (select oid from pg_catalog.pg_namespace
                    where nspname = 'pg_catalog')
order by 1;

SELECT p1.oid, p1.proname
FROM pg_proc AS p1
WHERE provolatile = 'i' AND proparallel = 'u';

SELECT *
FROM pg_cast c
WHERE castsource = 0 OR casttarget = 0 OR castcontext NOT IN ('e', 'a', 'i')
    OR castmethod NOT IN ('f', 'b' ,'i');

SELECT *
FROM pg_cast c
WHERE (castmethod = 'f' AND castfunc = 0)
   OR (castmethod IN ('b', 'i') AND castfunc <> 0);

SELECT *
FROM pg_cast c
WHERE castsource = casttarget AND castfunc = 0;

SELECT c.*
FROM pg_cast c, pg_proc p
WHERE c.castfunc = p.oid AND p.pronargs < 2 AND castsource = casttarget;

SELECT c.*
FROM pg_cast c, pg_proc p
WHERE c.castfunc = p.oid AND
    (p.pronargs < 1 OR p.pronargs > 3
     OR NOT (binary_coercible(c.castsource, p.proargtypes[0])
             OR (c.castsource = 'character'::regtype AND
                 p.proargtypes[0] = 'text'::regtype))
     OR NOT binary_coercible(p.prorettype, c.casttarget));

SELECT c.*
FROM pg_cast c, pg_proc p
WHERE c.castfunc = p.oid AND
    ((p.pronargs > 1 AND p.proargtypes[1] != 'int4'::regtype) OR
     (p.pronargs > 2 AND p.proargtypes[2] != 'bool'::regtype));

SELECT castsource::regtype, casttarget::regtype, castfunc, castcontext
FROM pg_cast c
WHERE c.castmethod = 'b' AND
    NOT EXISTS (SELECT 1 FROM pg_cast k
                WHERE k.castmethod = 'b' AND
                    k.castsource = c.casttarget AND
                    k.casttarget = c.castsource);

SELECT c.oid, c.conname
FROM pg_conversion as c
WHERE c.conproc = 0 OR
    pg_encoding_to_char(conforencoding) = '' OR
    pg_encoding_to_char(contoencoding) = '';

SELECT p.oid, p.proname, c.oid, c.conname
FROM pg_proc p, pg_conversion c
WHERE p.oid = c.conproc AND
    (p.prorettype != 'int4'::regtype OR p.proretset OR
     p.pronargs != 6 OR
     p.proargtypes[0] != 'int4'::regtype OR
     p.proargtypes[1] != 'int4'::regtype OR
     p.proargtypes[2] != 'cstring'::regtype OR
     p.proargtypes[3] != 'internal'::regtype OR
     p.proargtypes[4] != 'int4'::regtype OR
     p.proargtypes[5] != 'bool'::regtype);

SELECT c.oid, c.conname
FROM pg_conversion as c
WHERE condefault AND
    convert('ABC'::bytea, pg_encoding_to_char(conforencoding),
            pg_encoding_to_char(contoencoding)) != 'ABC';

SELECT o1.oid, o1.oprname
FROM pg_operator as o1
WHERE (o1.oprkind != 'b' AND o1.oprkind != 'l') OR
    o1.oprresult = 0 OR o1.oprcode = 0;

SELECT o1.oid, o1.oprname
FROM pg_operator as o1
WHERE (o1.oprleft = 0 and o1.oprkind != 'l') OR
    (o1.oprleft != 0 and o1.oprkind = 'l') OR
    o1.oprright = 0;

SELECT o1.oid, o1.oprcode, o2.oid, o2.oprcode
FROM pg_operator AS o1, pg_operator AS o2
WHERE o1.oid != o2.oid AND
    o1.oprname = o2.oprname AND
    o1.oprkind = o2.oprkind AND
    o1.oprleft = o2.oprleft AND
    o1.oprright = o2.oprright;

SELECT o1.oid, o1.oprcode, o2.oid, o2.oprcode
FROM pg_operator AS o1, pg_operator AS o2
WHERE o1.oprcom = o2.oid AND
    (o1.oprkind != 'b' OR
     o1.oprleft != o2.oprright OR
     o1.oprright != o2.oprleft OR
     o1.oprresult != o2.oprresult OR
     o1.oid != o2.oprcom);

SELECT o1.oid, o1.oprcode, o2.oid, o2.oprcode
FROM pg_operator AS o1, pg_operator AS o2
WHERE o1.oprnegate = o2.oid AND
    (o1.oprkind != o2.oprkind OR
     o1.oprleft != o2.oprleft OR
     o1.oprright != o2.oprright OR
     o1.oprresult != 'bool'::regtype OR
     o2.oprresult != 'bool'::regtype OR
     o1.oid != o2.oprnegate OR
     o1.oid = o2.oid);

SELECT DISTINCT o1.oprname AS op1, o2.oprname AS op2
FROM pg_operator o1, pg_operator o2
WHERE o1.oprcom = o2.oid AND o1.oprname <= o2.oprname
ORDER BY 1, 2;

SELECT DISTINCT o1.oprname AS op1, o2.oprname AS op2
FROM pg_operator o1, pg_operator o2
WHERE o1.oprnegate = o2.oid AND o1.oprname <= o2.oprname
ORDER BY 1, 2;

SELECT o1.oid, o1.oprname FROM pg_operator AS o1
WHERE (o1.oprcanmerge OR o1.oprcanhash) AND NOT
    (o1.oprkind = 'b' AND o1.oprresult = 'bool'::regtype AND o1.oprcom != 0);

SELECT o1.oid, o1.oprname, o2.oid, o2.oprname
FROM pg_operator AS o1, pg_operator AS o2
WHERE o1.oprcom = o2.oid AND
    (o1.oprcanmerge != o2.oprcanmerge OR
     o1.oprcanhash != o2.oprcanhash);

SELECT o1.oid, o1.oprname
FROM pg_operator AS o1
WHERE o1.oprcanmerge AND NOT EXISTS
  (SELECT 1 FROM pg_amop
   WHERE amopmethod = (SELECT oid FROM pg_am WHERE amname = 'btree') AND
         amopopr = o1.oid AND amopstrategy = 3);

SELECT o1.oid, o1.oprname, p.amopfamily
FROM pg_operator AS o1, pg_amop p
WHERE amopopr = o1.oid
  AND amopmethod = (SELECT oid FROM pg_am WHERE amname = 'btree')
  AND amopstrategy = 3
  AND NOT o1.oprcanmerge;

SELECT o1.oid, o1.oprname
FROM pg_operator AS o1
WHERE o1.oprcanhash AND NOT EXISTS
  (SELECT 1 FROM pg_amop
   WHERE amopmethod = (SELECT oid FROM pg_am WHERE amname = 'hash') AND
         amopopr = o1.oid AND amopstrategy = 1);

SELECT o1.oid, o1.oprname, p.amopfamily
FROM pg_operator AS o1, pg_amop p
WHERE amopopr = o1.oid
  AND amopmethod = (SELECT oid FROM pg_am WHERE amname = 'hash')
  AND NOT o1.oprcanhash;

SELECT o1.oid, o1.oprname, p1.oid, p1.proname
FROM pg_operator AS o1, pg_proc AS p1
WHERE o1.oprcode = p1.oid AND
    o1.oprkind = 'b' AND
    (p1.pronargs != 2
     OR NOT binary_coercible(p1.prorettype, o1.oprresult)
     OR NOT binary_coercible(o1.oprleft, p1.proargtypes[0])
     OR NOT binary_coercible(o1.oprright, p1.proargtypes[1]));

SELECT o1.oid, o1.oprname, p1.oid, p1.proname
FROM pg_operator AS o1, pg_proc AS p1
WHERE o1.oprcode = p1.oid AND
    o1.oprkind = 'l' AND
    (p1.pronargs != 1
     OR NOT binary_coercible(p1.prorettype, o1.oprresult)
     OR NOT binary_coercible(o1.oprright, p1.proargtypes[0])
     OR o1.oprleft != 0);

SELECT o1.oid, o1.oprname, p1.oid, p1.proname
FROM pg_operator AS o1, pg_proc AS p1
WHERE o1.oprcode = p1.oid AND
    (o1.oprcanmerge OR o1.oprcanhash) AND
    p1.provolatile = 'v';

SELECT o1.oid, o1.oprname, p2.oid, p2.proname
FROM pg_operator AS o1, pg_proc AS p2
WHERE o1.oprrest = p2.oid AND
    (o1.oprresult != 'bool'::regtype OR
     p2.prorettype != 'float8'::regtype OR p2.proretset OR
     p2.pronargs != 4 OR
     p2.proargtypes[0] != 'internal'::regtype OR
     p2.proargtypes[1] != 'oid'::regtype OR
     p2.proargtypes[2] != 'internal'::regtype OR
     p2.proargtypes[3] != 'int4'::regtype);

SELECT o1.oid, o1.oprname, p2.oid, p2.proname
FROM pg_operator AS o1, pg_proc AS p2
WHERE o1.oprjoin = p2.oid AND
    (o1.oprkind != 'b' OR o1.oprresult != 'bool'::regtype OR
     p2.prorettype != 'float8'::regtype OR p2.proretset OR
     p2.pronargs != 5 OR
     p2.proargtypes[0] != 'internal'::regtype OR
     p2.proargtypes[1] != 'oid'::regtype OR
     p2.proargtypes[2] != 'internal'::regtype OR
     p2.proargtypes[3] != 'int2'::regtype OR
     p2.proargtypes[4] != 'internal'::regtype);

SELECT o1.oid, o1.oprname
FROM pg_operator as o1 LEFT JOIN pg_description as d
     ON o1.tableoid = d.classoid and o1.oid = d.objoid and d.objsubid = 0
WHERE d.classoid IS NULL AND o1.oid <= 9999;

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
SELECT * FROM funcdescs
  WHERE prodesc IS DISTINCT FROM expecteddesc
    AND oprdesc NOT LIKE 'deprecated%'
    AND prodesc IS DISTINCT FROM oprdesc;

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

SELECT o1.oid, o1.oprcode, o2.oid, o2.oprcode
FROM pg_operator AS o1, pg_operator AS o2, pg_proc AS p1, pg_proc AS p2
WHERE o1.oprcom = o2.oid AND p1.oid = o1.oprcode AND p2.oid = o2.oprcode AND
    (p1.provolatile != p2.provolatile OR
     p1.proleakproof != p2.proleakproof);

SELECT o1.oid, o1.oprcode, o2.oid, o2.oprcode
FROM pg_operator AS o1, pg_operator AS o2, pg_proc AS p1, pg_proc AS p2
WHERE o1.oprnegate = o2.oid AND p1.oid = o1.oprcode AND p2.oid = o2.oprcode AND
    (p1.provolatile != p2.provolatile OR
     p1.proleakproof != p2.proleakproof);

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

SELECT ctid, aggfnoid::oid
FROM pg_aggregate as a
WHERE aggfnoid = 0 OR aggtransfn = 0 OR
    aggkind NOT IN ('n', 'o', 'h') OR
    aggnumdirectargs < 0 OR
    (aggkind = 'n' AND aggnumdirectargs > 0) OR
    aggfinalmodify NOT IN ('r', 's', 'w') OR
    aggmfinalmodify NOT IN ('r', 's', 'w') OR
    aggtranstype = 0 OR aggtransspace < 0 OR aggmtransspace < 0;

SELECT a.aggfnoid::oid, p.proname
FROM pg_aggregate as a, pg_proc as p
WHERE a.aggfnoid = p.oid AND
    (p.prokind != 'a' OR p.proretset OR p.pronargs < a.aggnumdirectargs);

SELECT oid, proname
FROM pg_proc as p
WHERE p.prokind = 'a' AND
    NOT EXISTS (SELECT 1 FROM pg_aggregate a WHERE a.aggfnoid = p.oid);

SELECT a.aggfnoid::oid, p.proname
FROM pg_aggregate as a, pg_proc as p
WHERE a.aggfnoid = p.oid AND
    a.aggfinalfn = 0 AND p.prorettype != a.aggtranstype;

SELECT a.aggfnoid::oid, p.proname, ptr.oid, ptr.proname
FROM pg_aggregate AS a, pg_proc AS p, pg_proc AS ptr
WHERE a.aggfnoid = p.oid AND
    a.aggtransfn = ptr.oid AND
    (ptr.proretset
     OR NOT (ptr.pronargs =
             CASE WHEN a.aggkind = 'n' THEN p.pronargs + 1
             ELSE greatest(p.pronargs - a.aggnumdirectargs, 1) + 1 END)
     OR NOT binary_coercible(ptr.prorettype, a.aggtranstype)
     OR NOT binary_coercible(a.aggtranstype, ptr.proargtypes[0])
     OR (p.pronargs > 0 AND
         NOT binary_coercible(p.proargtypes[0], ptr.proargtypes[1]))
     OR (p.pronargs > 1 AND
         NOT binary_coercible(p.proargtypes[1], ptr.proargtypes[2]))
     OR (p.pronargs > 2 AND
         NOT binary_coercible(p.proargtypes[2], ptr.proargtypes[3]))
     OR (p.pronargs > 3 AND
         NOT binary_coercible(p.proargtypes[3], ptr.proargtypes[4]))
     -- we could carry the check further, but 4 args is enough for now
     OR (p.pronargs > 4)
    );

SELECT a.aggfnoid::oid, p.proname, pfn.oid, pfn.proname
FROM pg_aggregate AS a, pg_proc AS p, pg_proc AS pfn
WHERE a.aggfnoid = p.oid AND
    a.aggfinalfn = pfn.oid AND
    (pfn.proretset OR
     NOT binary_coercible(pfn.prorettype, p.prorettype) OR
     NOT binary_coercible(a.aggtranstype, pfn.proargtypes[0]) OR
     CASE WHEN a.aggfinalextra THEN pfn.pronargs != p.pronargs + 1
          ELSE pfn.pronargs != a.aggnumdirectargs + 1 END
     OR (pfn.pronargs > 1 AND
         NOT binary_coercible(p.proargtypes[0], pfn.proargtypes[1]))
     OR (pfn.pronargs > 2 AND
         NOT binary_coercible(p.proargtypes[1], pfn.proargtypes[2]))
     OR (pfn.pronargs > 3 AND
         NOT binary_coercible(p.proargtypes[2], pfn.proargtypes[3]))
     -- we could carry the check further, but 4 args is enough for now
     OR (pfn.pronargs > 4)
    );

SELECT a.aggfnoid::oid, p.proname, ptr.oid, ptr.proname
FROM pg_aggregate AS a, pg_proc AS p, pg_proc AS ptr
WHERE a.aggfnoid = p.oid AND
    a.aggtransfn = ptr.oid AND ptr.proisstrict AND
    a.agginitval IS NULL AND
    NOT binary_coercible(p.proargtypes[0], a.aggtranstype);

SELECT ctid, aggfnoid::oid
FROM pg_aggregate as a
WHERE aggmtranstype != 0 AND
    (aggmtransfn = 0 OR aggminvtransfn = 0);

SELECT ctid, aggfnoid::oid
FROM pg_aggregate as a
WHERE aggmtranstype = 0 AND
    (aggmtransfn != 0 OR aggminvtransfn != 0 OR aggmfinalfn != 0 OR
     aggmtransspace != 0 OR aggminitval IS NOT NULL);

SELECT a.aggfnoid::oid, p.proname
FROM pg_aggregate as a, pg_proc as p
WHERE a.aggfnoid = p.oid AND
    a.aggmtransfn != 0 AND
    a.aggmfinalfn = 0 AND p.prorettype != a.aggmtranstype;

SELECT a.aggfnoid::oid, p.proname, ptr.oid, ptr.proname
FROM pg_aggregate AS a, pg_proc AS p, pg_proc AS ptr
WHERE a.aggfnoid = p.oid AND
    a.aggmtransfn = ptr.oid AND
    (ptr.proretset
     OR NOT (ptr.pronargs =
             CASE WHEN a.aggkind = 'n' THEN p.pronargs + 1
             ELSE greatest(p.pronargs - a.aggnumdirectargs, 1) + 1 END)
     OR NOT binary_coercible(ptr.prorettype, a.aggmtranstype)
     OR NOT binary_coercible(a.aggmtranstype, ptr.proargtypes[0])
     OR (p.pronargs > 0 AND
         NOT binary_coercible(p.proargtypes[0], ptr.proargtypes[1]))
     OR (p.pronargs > 1 AND
         NOT binary_coercible(p.proargtypes[1], ptr.proargtypes[2]))
     OR (p.pronargs > 2 AND
         NOT binary_coercible(p.proargtypes[2], ptr.proargtypes[3]))
     -- we could carry the check further, but 3 args is enough for now
     OR (p.pronargs > 3)
    );

SELECT a.aggfnoid::oid, p.proname, ptr.oid, ptr.proname
FROM pg_aggregate AS a, pg_proc AS p, pg_proc AS ptr
WHERE a.aggfnoid = p.oid AND
    a.aggminvtransfn = ptr.oid AND
    (ptr.proretset
     OR NOT (ptr.pronargs =
             CASE WHEN a.aggkind = 'n' THEN p.pronargs + 1
             ELSE greatest(p.pronargs - a.aggnumdirectargs, 1) + 1 END)
     OR NOT binary_coercible(ptr.prorettype, a.aggmtranstype)
     OR NOT binary_coercible(a.aggmtranstype, ptr.proargtypes[0])
     OR (p.pronargs > 0 AND
         NOT binary_coercible(p.proargtypes[0], ptr.proargtypes[1]))
     OR (p.pronargs > 1 AND
         NOT binary_coercible(p.proargtypes[1], ptr.proargtypes[2]))
     OR (p.pronargs > 2 AND
         NOT binary_coercible(p.proargtypes[2], ptr.proargtypes[3]))
     -- we could carry the check further, but 3 args is enough for now
     OR (p.pronargs > 3)
    );

SELECT a.aggfnoid::oid, p.proname, pfn.oid, pfn.proname
FROM pg_aggregate AS a, pg_proc AS p, pg_proc AS pfn
WHERE a.aggfnoid = p.oid AND
    a.aggmfinalfn = pfn.oid AND
    (pfn.proretset OR
     NOT binary_coercible(pfn.prorettype, p.prorettype) OR
     NOT binary_coercible(a.aggmtranstype, pfn.proargtypes[0]) OR
     CASE WHEN a.aggmfinalextra THEN pfn.pronargs != p.pronargs + 1
          ELSE pfn.pronargs != a.aggnumdirectargs + 1 END
     OR (pfn.pronargs > 1 AND
         NOT binary_coercible(p.proargtypes[0], pfn.proargtypes[1]))
     OR (pfn.pronargs > 2 AND
         NOT binary_coercible(p.proargtypes[1], pfn.proargtypes[2]))
     OR (pfn.pronargs > 3 AND
         NOT binary_coercible(p.proargtypes[2], pfn.proargtypes[3]))
     -- we could carry the check further, but 4 args is enough for now
     OR (pfn.pronargs > 4)
    );

SELECT a.aggfnoid::oid, p.proname, ptr.oid, ptr.proname
FROM pg_aggregate AS a, pg_proc AS p, pg_proc AS ptr
WHERE a.aggfnoid = p.oid AND
    a.aggmtransfn = ptr.oid AND ptr.proisstrict AND
    a.aggminitval IS NULL AND
    NOT binary_coercible(p.proargtypes[0], a.aggmtranstype);

SELECT a.aggfnoid::oid, p.proname, ptr.oid, ptr.proname, iptr.oid, iptr.proname
FROM pg_aggregate AS a, pg_proc AS p, pg_proc AS ptr, pg_proc AS iptr
WHERE a.aggfnoid = p.oid AND
    a.aggmtransfn = ptr.oid AND
    a.aggminvtransfn = iptr.oid AND
    ptr.proisstrict != iptr.proisstrict;

SELECT a.aggfnoid, p.proname
FROM pg_aggregate as a, pg_proc as p
WHERE a.aggcombinefn = p.oid AND
    (p.pronargs != 2 OR
     p.prorettype != p.proargtypes[0] OR
     p.prorettype != p.proargtypes[1] OR
     NOT binary_coercible(a.aggtranstype, p.proargtypes[0]));

SELECT a.aggfnoid, p.proname
FROM pg_aggregate as a, pg_proc as p
WHERE a.aggcombinefn = p.oid AND
    a.aggtranstype = 'internal'::regtype AND p.proisstrict;

SELECT aggfnoid, aggtranstype, aggserialfn, aggdeserialfn
FROM pg_aggregate
WHERE (aggserialfn != 0 OR aggdeserialfn != 0)
  AND (aggtranstype != 'internal'::regtype OR aggcombinefn = 0 OR
       aggserialfn = 0 OR aggdeserialfn = 0);

SELECT a.aggfnoid, p.proname
FROM pg_aggregate as a, pg_proc as p
WHERE a.aggserialfn = p.oid AND
    (p.prorettype != 'bytea'::regtype OR p.pronargs != 1 OR
     p.proargtypes[0] != 'internal'::regtype OR
     NOT p.proisstrict);

SELECT a.aggfnoid, p.proname
FROM pg_aggregate as a, pg_proc as p
WHERE a.aggdeserialfn = p.oid AND
    (p.prorettype != 'internal'::regtype OR p.pronargs != 2 OR
     p.proargtypes[0] != 'bytea'::regtype OR
     p.proargtypes[1] != 'internal'::regtype OR
     NOT p.proisstrict);

SELECT a.aggfnoid, a.aggcombinefn, a.aggserialfn, a.aggdeserialfn,
       b.aggfnoid, b.aggcombinefn, b.aggserialfn, b.aggdeserialfn
FROM
    pg_aggregate a, pg_aggregate b
WHERE
    a.aggfnoid < b.aggfnoid AND a.aggtransfn = b.aggtransfn AND
    (a.aggcombinefn != b.aggcombinefn OR a.aggserialfn != b.aggserialfn
     OR a.aggdeserialfn != b.aggdeserialfn);

SELECT DISTINCT proname, oprname
FROM pg_operator AS o, pg_aggregate AS a, pg_proc AS p
WHERE a.aggfnoid = p.oid AND a.aggsortop = o.oid
ORDER BY 1, 2;

SELECT a.aggfnoid::oid, o.oid
FROM pg_operator AS o, pg_aggregate AS a, pg_proc AS p
WHERE a.aggfnoid = p.oid AND a.aggsortop = o.oid AND
    (oprkind != 'b' OR oprresult != 'boolean'::regtype
     OR oprleft != p.proargtypes[0] OR oprright != p.proargtypes[0]);

SELECT a.aggfnoid::oid, o.oid
FROM pg_operator AS o, pg_aggregate AS a, pg_proc AS p
WHERE a.aggfnoid = p.oid AND a.aggsortop = o.oid AND
    NOT EXISTS(SELECT 1 FROM pg_amop
               WHERE amopmethod = (SELECT oid FROM pg_am WHERE amname = 'btree')
                     AND amopopr = o.oid
                     AND amoplefttype = o.oprleft
                     AND amoprighttype = o.oprright);

SELECT DISTINCT proname, oprname, amopstrategy
FROM pg_operator AS o, pg_aggregate AS a, pg_proc AS p,
     pg_amop as ao
WHERE a.aggfnoid = p.oid AND a.aggsortop = o.oid AND
    amopopr = o.oid AND
    amopmethod = (SELECT oid FROM pg_am WHERE amname = 'btree')
ORDER BY 1, 2;

SELECT p1.oid::regprocedure, p2.oid::regprocedure
FROM pg_proc AS p1, pg_proc AS p2
WHERE p1.oid < p2.oid AND p1.proname = p2.proname AND
    p1.prokind = 'a' AND p2.prokind = 'a' AND
    array_dims(p1.proargtypes) != array_dims(p2.proargtypes)
ORDER BY 1;

SELECT oid, proname
FROM pg_proc AS p
WHERE prokind = 'a' AND proargdefaults IS NOT NULL;

SELECT p.oid, proname
FROM pg_proc AS p JOIN pg_aggregate AS a ON a.aggfnoid = p.oid
WHERE prokind = 'a' AND provariadic != 0 AND a.aggkind = 'n';

SELECT f.oid
FROM pg_opfamily as f
WHERE f.opfmethod = 0 OR f.opfnamespace = 0;

SELECT oid, opfname FROM pg_opfamily f
WHERE NOT EXISTS (SELECT 1 FROM pg_opclass WHERE opcfamily = f.oid);

SELECT c1.oid
FROM pg_opclass AS c1
WHERE c1.opcmethod = 0 OR c1.opcnamespace = 0 OR c1.opcfamily = 0
    OR c1.opcintype = 0;

SELECT c1.oid, f1.oid
FROM pg_opclass AS c1, pg_opfamily AS f1
WHERE c1.opcfamily = f1.oid AND c1.opcmethod != f1.opfmethod;

SELECT c1.oid, c2.oid
FROM pg_opclass AS c1, pg_opclass AS c2
WHERE c1.oid != c2.oid AND
    c1.opcmethod = c2.opcmethod AND c1.opcintype = c2.opcintype AND
    c1.opcdefault AND c2.opcdefault;

SELECT oid, opcname FROM pg_opclass WHERE NOT amvalidate(oid);

SELECT a1.oid, a1.amname
FROM pg_am AS a1
WHERE a1.amhandler = 0;

SELECT a1.oid, a1.amname, p1.oid, p1.proname
FROM pg_am AS a1, pg_proc AS p1
WHERE p1.oid = a1.amhandler AND a1.amtype = 'i' AND
    (p1.prorettype != 'index_am_handler'::regtype
     OR p1.proretset
     OR p1.pronargs != 1
     OR p1.proargtypes[0] != 'internal'::regtype);

SELECT a1.oid, a1.amname, p1.oid, p1.proname
FROM pg_am AS a1, pg_proc AS p1
WHERE p1.oid = a1.amhandler AND a1.amtype = 't' AND
    (p1.prorettype != 'table_am_handler'::regtype
     OR p1.proretset
     OR p1.pronargs != 1
     OR p1.proargtypes[0] != 'internal'::regtype);

SELECT a1.amopfamily, a1.amopstrategy
FROM pg_amop as a1
WHERE a1.amopfamily = 0 OR a1.amoplefttype = 0 OR a1.amoprighttype = 0
    OR a1.amopopr = 0 OR a1.amopmethod = 0 OR a1.amopstrategy < 1;

SELECT a1.amopfamily, a1.amopstrategy
FROM pg_amop as a1
WHERE NOT ((a1.amoppurpose = 's' AND a1.amopsortfamily = 0) OR
           (a1.amoppurpose = 'o' AND a1.amopsortfamily <> 0));

SELECT a1.oid, f1.oid
FROM pg_amop AS a1, pg_opfamily AS f1
WHERE a1.amopfamily = f1.oid AND a1.amopmethod != f1.opfmethod;

SELECT DISTINCT amopmethod, amopstrategy, oprname
FROM pg_amop a1 LEFT JOIN pg_operator o1 ON amopopr = o1.oid
ORDER BY 1, 2, 3;

SELECT a1.amopfamily, a1.amopopr, o1.oid, o1.oprname
FROM pg_amop AS a1, pg_operator AS o1
WHERE a1.amopopr = o1.oid AND a1.amoppurpose = 's' AND
    (o1.oprrest = 0 OR o1.oprjoin = 0);

SELECT c1.opcname, c1.opcfamily
FROM pg_opclass AS c1
WHERE NOT EXISTS(SELECT 1 FROM pg_amop AS a1
                 WHERE a1.amopfamily = c1.opcfamily
                   AND binary_coercible(c1.opcintype, a1.amoplefttype));

SELECT a1.amopfamily, a1.amopstrategy, a1.amopopr
FROM pg_amop AS a1
WHERE NOT EXISTS(SELECT 1 FROM pg_opclass AS c1
                 WHERE c1.opcfamily = a1.amopfamily
                   AND binary_coercible(c1.opcintype, a1.amoplefttype));

SELECT a1.amopfamily, a1.amopopr, o1.oprname, p1.prosrc
FROM pg_amop AS a1, pg_operator AS o1, pg_proc AS p1
WHERE a1.amopopr = o1.oid AND o1.oprcode = p1.oid AND
    a1.amoplefttype = a1.amoprighttype AND
    p1.provolatile != 'i';

SELECT a1.amopfamily, a1.amopopr, o1.oprname, p1.prosrc
FROM pg_amop AS a1, pg_operator AS o1, pg_proc AS p1
WHERE a1.amopopr = o1.oid AND o1.oprcode = p1.oid AND
    a1.amoplefttype != a1.amoprighttype AND
    p1.provolatile = 'v';

SELECT a1.amprocfamily, a1.amprocnum
FROM pg_amproc as a1
WHERE a1.amprocfamily = 0 OR a1.amproclefttype = 0 OR a1.amprocrighttype = 0
    OR a1.amprocnum < 0 OR a1.amproc = 0;

SELECT a1.amprocfamily, a1.amproc, p1.prosrc
FROM pg_amproc AS a1, pg_proc AS p1
WHERE a1.amproc = p1.oid AND
    a1.amproclefttype = a1.amprocrighttype AND
    p1.provolatile != 'i';

SELECT a1.amprocfamily, a1.amproc, p1.prosrc
FROM pg_amproc AS a1, pg_proc AS p1
WHERE a1.amproc = p1.oid AND
    a1.amproclefttype != a1.amprocrighttype AND
    p1.provolatile = 'v';

SELECT amp.amproc::regproc AS proc, opf.opfname AS opfamily_name,
       opc.opcname AS opclass_name, opc.opcintype::regtype AS opcintype
FROM pg_am AS am
JOIN pg_opclass AS opc ON opc.opcmethod = am.oid
JOIN pg_opfamily AS opf ON opc.opcfamily = opf.oid
LEFT JOIN pg_amproc AS amp ON amp.amprocfamily = opf.oid AND
    amp.amproclefttype = opc.opcintype AND amp.amprocnum = 4
WHERE am.amname = 'btree' AND
    amp.amproc IS DISTINCT FROM 'btequalimage'::regproc
ORDER BY 1, 2, 3;

SELECT indexrelid, indrelid
FROM pg_index
WHERE indexrelid = 0 OR indrelid = 0 OR
      indnatts <= 0 OR indnatts > 32;

SELECT indexrelid, indrelid
FROM pg_index
WHERE array_lower(indkey, 1) != 0 OR array_upper(indkey, 1) != indnatts-1 OR
    array_lower(indclass, 1) != 0 OR array_upper(indclass, 1) != indnatts-1 OR
    array_lower(indcollation, 1) != 0 OR array_upper(indcollation, 1) != indnatts-1 OR
    array_lower(indoption, 1) != 0 OR array_upper(indoption, 1) != indnatts-1;

SELECT indexrelid::regclass, indrelid::regclass, attname, atttypid::regtype, opcname
FROM (SELECT indexrelid, indrelid, unnest(indkey) as ikey,
             unnest(indclass) as iclass, unnest(indcollation) as icoll
      FROM pg_index) ss,
      pg_attribute a,
      pg_opclass opc
WHERE a.attrelid = indrelid AND a.attnum = ikey AND opc.oid = iclass AND
      (NOT binary_coercible(atttypid, opcintype) OR icoll != attcollation);

SELECT indexrelid::regclass, indrelid::regclass, attname, atttypid::regtype, opcname
FROM (SELECT indexrelid, indrelid, unnest(indkey) as ikey,
             unnest(indclass) as iclass, unnest(indcollation) as icoll
      FROM pg_index
      WHERE indrelid < 16384) ss,
      pg_attribute a,
      pg_opclass opc
WHERE a.attrelid = indrelid AND a.attnum = ikey AND opc.oid = iclass AND
      (opcintype != atttypid OR icoll != attcollation)
ORDER BY 1;

SELECT relname, attname, attcollation
FROM pg_class c, pg_attribute a
WHERE c.oid = attrelid AND c.oid < 16384 AND
    c.relkind != 'v' AND  -- we don't care about columns in views
    attcollation != 0 AND
    attcollation != (SELECT oid FROM pg_collation WHERE collname = 'C');

SELECT indexrelid::regclass, indrelid::regclass, iclass, icoll
FROM (SELECT indexrelid, indrelid,
             unnest(indclass) as iclass, unnest(indcollation) as icoll
      FROM pg_index
      WHERE indrelid < 16384) ss
WHERE icoll != 0 AND
    icoll != (SELECT oid FROM pg_collation WHERE collname = 'C');

WITH objects (type, name, args) AS (VALUES
    ('table', '{addr_nsp, gentable}'::text[], '{}'::text[]),
    ('table', '{addr_nsp, parttable}'::text[], '{}'::text[]),
    ('index', '{addr_nsp, gentable_pkey}', '{}'),
    ('index', '{addr_nsp, parttable_pkey}', '{}'),
    ('sequence', '{addr_nsp, gentable_a_seq}', '{}'),
    -- toast table
    ('view', '{addr_nsp, genview}', '{}'),
    ('materialized view', '{addr_nsp, genmatview}', '{}'),
    ('foreign table', '{addr_nsp, genftable}', '{}'),
    ('table column', '{addr_nsp, gentable, b}', '{}'),
    ('foreign table column', '{addr_nsp, genftable, a}', '{}'),
    ('aggregate', '{addr_nsp, genaggr}', '{int4}'),
    ('function', '{pg_catalog, pg_identify_object}', '{pg_catalog.oid, pg_catalog.oid, int4}'),
    ('procedure', '{addr_nsp, proc}', '{int4}'),
    ('type', '{pg_catalog._int4}', '{}'),
    ('type', '{addr_nsp.gendomain}', '{}'),
    ('type', '{addr_nsp.gencomptype}', '{}'),
    ('type', '{addr_nsp.genenum}', '{}'),
    ('cast', '{int8}', '{int4}'),
    ('collation', '{default}', '{}'),
    ('table constraint', '{addr_nsp, gentable, a_chk}', '{}'),
    ('domain constraint', '{addr_nsp.gendomain}', '{domconstr}'),
    ('conversion', '{pg_catalog, koi8_r_to_mic}', '{}'),
    ('default value', '{addr_nsp, gentable, b}', '{}'),
    ('language', '{plpgsql}', '{}'),
    -- large object
    ('operator', '{+}', '{int4, int4}'),
    ('operator class', '{btree, int4_ops}', '{}'),
    ('operator family', '{btree, integer_ops}', '{}'),
    ('operator of access method', '{btree,integer_ops,1}', '{integer,integer}'),
    ('function of access method', '{btree,integer_ops,2}', '{integer,integer}'),
    ('rule', '{addr_nsp, genview, _RETURN}', '{}'),
    ('trigger', '{addr_nsp, gentable, t}', '{}'),
    ('schema', '{addr_nsp}', '{}'),
    ('text search parser', '{addr_ts_prs}', '{}'),
    ('text search dictionary', '{addr_ts_dict}', '{}'),
    ('text search template', '{addr_ts_temp}', '{}'),
    ('text search configuration', '{addr_ts_conf}', '{}'),
    ('role', '{regress_addr_user}', '{}'),
    -- database
    -- tablespace
    ('foreign-data wrapper', '{addr_fdw}', '{}'),
    ('server', '{addr_fserv}', '{}'),
    ('user mapping', '{regress_addr_user}', '{integer}'),
    ('default acl', '{regress_addr_user,public}', '{r}'),
    ('default acl', '{regress_addr_user}', '{r}'),
    -- extension
    -- event trigger
    ('policy', '{addr_nsp, gentable, genpol}', '{}'),
    ('transform', '{int}', '{sql}'),
    ('access method', '{btree}', '{}'),
    ('publication', '{addr_pub}', '{}'),
    ('publication namespace', '{addr_nsp}', '{addr_pub_schema}'),
    ('publication relation', '{addr_nsp, gentable}', '{addr_pub}'),
    ('subscription', '{regress_addr_sub}', '{}'),
    ('statistics object', '{addr_nsp, gentable_stat}', '{}')
  )
SELECT (pg_identify_object(addr1.classid, addr1.objid, addr1.objsubid)).*,
        -- test roundtrip through pg_identify_object_as_address
        ROW(pg_identify_object(addr1.classid, addr1.objid, addr1.objsubid)) =
          ROW(pg_identify_object(addr2.classid, addr2.objid, addr2.objsubid)) AS roundtrip
FROM objects,
     pg_get_object_address(type, name, args) AS addr1,
     pg_identify_object_as_address(classid, objid, objsubid) AS ioa (typ, nms, args),
     pg_get_object_address(typ, nms, ioa.args) AS addr2
ORDER BY addr1.classid, addr1.objid, addr1.objsubid;

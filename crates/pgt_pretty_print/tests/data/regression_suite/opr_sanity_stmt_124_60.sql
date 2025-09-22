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

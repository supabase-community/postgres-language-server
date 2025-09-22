SELECT c1.oid, c2.oid
FROM pg_opclass AS c1, pg_opclass AS c2
WHERE c1.oid != c2.oid AND
    c1.opcmethod = c2.opcmethod AND c1.opcintype = c2.opcintype AND
    c1.opcdefault AND c2.opcdefault;

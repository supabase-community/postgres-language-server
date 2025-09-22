SELECT a1.amopfamily, a1.amopopr, o1.oprname, p1.prosrc
FROM pg_amop AS a1, pg_operator AS o1, pg_proc AS p1
WHERE a1.amopopr = o1.oid AND o1.oprcode = p1.oid AND
    a1.amoplefttype = a1.amoprighttype AND
    p1.provolatile != 'i';

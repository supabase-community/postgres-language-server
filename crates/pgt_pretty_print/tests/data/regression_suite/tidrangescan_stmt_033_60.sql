SELECT t.ctid,t2.c FROM tidrangescan t,
LATERAL (SELECT count(*) c FROM tidrangescan t2 WHERE t2.ctid <= t.ctid) t2
WHERE t.ctid < '(1,0)';

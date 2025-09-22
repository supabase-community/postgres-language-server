CREATE POLICY p2 ON s2 USING (x in (select a from s1 where b like '%22%'));

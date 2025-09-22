INSERT INTO mcv_lists_uuid (a, b, c)
     SELECT
         fipshash(mod(i,100)::text)::uuid,
         fipshash(mod(i,50)::text)::uuid,
         fipshash(mod(i,25)::text)::uuid
     FROM generate_series(1,5000) s(i);

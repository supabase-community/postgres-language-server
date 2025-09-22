insert into upsert values(1, 'val') on conflict (key) do update set val = 'not seen';

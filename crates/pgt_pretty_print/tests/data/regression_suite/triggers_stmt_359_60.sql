insert into upsert values(1, 'black') on conflict (key) do update set color = 'updated ' || upsert.color;

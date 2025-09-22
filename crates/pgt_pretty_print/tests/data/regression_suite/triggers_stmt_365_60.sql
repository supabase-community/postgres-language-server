insert into upsert values(7, 'pink') on conflict (key) do update set color = 'updated ' || upsert.color;

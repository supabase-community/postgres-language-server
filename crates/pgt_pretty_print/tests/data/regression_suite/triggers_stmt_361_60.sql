insert into upsert values(3, 'orange') on conflict (key) do update set color = 'updated ' || upsert.color;

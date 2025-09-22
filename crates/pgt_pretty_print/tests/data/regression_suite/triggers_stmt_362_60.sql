insert into upsert values(4, 'green') on conflict (key) do update set color = 'updated ' || upsert.color;

insert into insertconflictview values(0, 'Crowberry') on conflict (lower(fruit), key, lower(fruit), key) do nothing;

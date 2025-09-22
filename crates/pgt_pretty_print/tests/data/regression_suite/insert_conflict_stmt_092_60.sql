insert into insertconflictview as t values (23, 'Blackberry') on conflict (key) where fruit like '%berry' and t.fruit = 'inconsequential' do nothing;

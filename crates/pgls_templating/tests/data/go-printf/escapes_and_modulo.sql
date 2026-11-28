select 7 %% 2, '100%%', n %% %d from t;
-- a lone percent sign that starts no directive is SQL, which Go would mangle
select 7 % 2, 8, 9 %;
select '%z', '%[x]d', '%[1d', '%Y', '%[0]d', '%';
-- before an operator character, the padding goes first: Go writes `%++`, one operator
select a%%++c;

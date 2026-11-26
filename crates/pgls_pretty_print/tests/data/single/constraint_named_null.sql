-- A named NULL constraint keeps its name, as every other named constraint does.
create table t (a int not null constraint c null, b int constraint d default 1, c int constraint e generated always as (b) stored);

create table ledger (
  id int primary key,
  balance numeric,
  updated_at timestamptz
);

create table adjustments (
  id int,
  delta numeric,
  seen_at timestamptz
);

update ledger
set (balance, updated_at) = (balance + delta, seen_at)
from adjustments
where ledger.id = adjustments.id;

insert into ledger as l (id, balance, updated_at)
values (1, 10, now())
on conflict (id) do update
set (balance, updated_at) = (excluded.balance + l.balance, excluded.updated_at);

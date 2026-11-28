-- the printer writes `limit` before `offset`, which would swap the arguments `{}`: the
-- statement stays as written
select * from t offset {} limit {};
-- in the printer's order, it is formatted
select   * from t limit {} offset {};

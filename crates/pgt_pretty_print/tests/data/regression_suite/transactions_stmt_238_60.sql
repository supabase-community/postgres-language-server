create or replace function max_xacttest() returns smallint language plpgsql as
'begin return max(a) from xacttest; end' stable;

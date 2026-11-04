select count(*) filter (where status = 'active') from users;

select count(*) filter (where status = 'active' and created_at > now() - interval '30 days' and deleted_at is null) as active_recently from users;

select sum(amount) filter (where kind = 'credit') - sum(amount) filter (where kind = 'debit') as balance from ledger group by account_id;

select percentile_cont(0.5) within group (order by response_time_in_milliseconds) filter (where status_code < 500) as median_response_time from requests;

select array_agg(distinct name order by name) filter (where name is not null) over (partition by team_id) from members;

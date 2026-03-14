CREATE OR REPLACE VIEW profile_view AS WITH user_cte AS (SELECT * FROM accounts) SELECT * FROM user_cte;

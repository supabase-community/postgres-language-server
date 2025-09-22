SELECT id,lag(id) OVER(), count(*) OVER(), generate_series(1,3) FROM few;

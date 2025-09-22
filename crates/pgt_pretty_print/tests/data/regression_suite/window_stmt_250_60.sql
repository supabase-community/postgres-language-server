SELECT rank() OVER (ORDER BY rank() OVER (ORDER BY random()));

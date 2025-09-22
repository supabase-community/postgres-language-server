PREPARE foo AS
  SELECT id, keywords, title, body, created
  FROM articles
  GROUP BY id;

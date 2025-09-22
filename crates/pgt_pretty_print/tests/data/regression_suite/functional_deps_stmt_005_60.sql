SELECT id, keywords, title, body, created
FROM articles
GROUP BY keywords;

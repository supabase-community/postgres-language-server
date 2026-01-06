CREATE TEMP TABLE articles (
    id int CONSTRAINT articles_pkey PRIMARY KEY,
    keywords text,
    title text UNIQUE NOT NULL,
    body text UNIQUE,
    created date
);

CREATE TEMP TABLE articles_in_category (
    article_id int,
    category_id int,
    changed date,
    PRIMARY KEY (article_id, category_id)
);

SELECT id, keywords, title, body, created
FROM articles
GROUP BY id;

SELECT id, keywords, title, body, created
FROM articles
GROUP BY title;

SELECT id, keywords, title, body, created
FROM articles
GROUP BY body;

SELECT id, keywords, title, body, created
FROM articles
GROUP BY keywords;

SELECT a.id, a.keywords, a.title, a.body, a.created
FROM articles AS a, articles_in_category AS aic
WHERE a.id = aic.article_id AND aic.category_id in (14,62,70,53,138)
GROUP BY a.id;

SELECT a.id, a.keywords, a.title, a.body, a.created
FROM articles AS a, articles_in_category AS aic
WHERE a.id = aic.article_id AND aic.category_id in (14,62,70,53,138)
GROUP BY aic.article_id, aic.category_id;

SELECT a.id, a.keywords, a.title, a.body, a.created
FROM articles AS a JOIN articles_in_category AS aic ON a.id = aic.article_id
WHERE aic.category_id in (14,62,70,53,138)
GROUP BY a.id;

SELECT a.id, a.keywords, a.title, a.body, a.created
FROM articles AS a JOIN articles_in_category AS aic ON a.id = aic.article_id
WHERE aic.category_id in (14,62,70,53,138)
GROUP BY aic.article_id, aic.category_id;

SELECT aic.changed
FROM articles AS a JOIN articles_in_category AS aic ON a.id = aic.article_id
WHERE aic.category_id in (14,62,70,53,138)
GROUP BY aic.category_id, aic.article_id;

SELECT aic.changed
FROM articles AS a JOIN articles_in_category AS aic ON a.id = aic.article_id
WHERE aic.category_id in (14,62,70,53,138)
GROUP BY aic.article_id;

CREATE TEMP TABLE products (product_id int, name text, price numeric);

CREATE TEMP TABLE sales (product_id int, units int);

SELECT product_id, p.name, (sum(s.units) * p.price) AS sales
    FROM products p LEFT JOIN sales s USING (product_id)
    GROUP BY product_id, p.name, p.price;

SELECT product_id, p.name, (sum(s.units) * p.price) AS sales
    FROM products p LEFT JOIN sales s USING (product_id)
    GROUP BY product_id;

ALTER TABLE products ADD PRIMARY KEY (product_id);

SELECT product_id, p.name, (sum(s.units) * p.price) AS sales
    FROM products p LEFT JOIN sales s USING (product_id)
    GROUP BY product_id;

CREATE TEMP TABLE node (
    nid SERIAL,
    vid integer NOT NULL default '0',
    type varchar(32) NOT NULL default '',
    title varchar(128) NOT NULL default '',
    uid integer NOT NULL default '0',
    status integer NOT NULL default '1',
    created integer NOT NULL default '0',
    -- snip
    PRIMARY KEY (nid, vid)
);

CREATE TEMP TABLE users (
    uid integer NOT NULL default '0',
    name varchar(60) NOT NULL default '',
    pass varchar(32) NOT NULL default '',
    -- snip
    PRIMARY KEY (uid),
    UNIQUE (name)
);

SELECT u.uid, u.name FROM node n
INNER JOIN users u ON u.uid = n.uid
WHERE n.type = 'blog' AND n.status = 1
GROUP BY u.uid, u.name;

SELECT u.uid, u.name FROM node n
INNER JOIN users u ON u.uid = n.uid
WHERE n.type = 'blog' AND n.status = 1
GROUP BY u.uid;

CREATE TEMP VIEW fdv1 AS
SELECT id, keywords, title, body, created
FROM articles
GROUP BY body;

CREATE TEMP VIEW fdv1 AS
SELECT id, keywords, title, body, created
FROM articles
GROUP BY id;

ALTER TABLE articles DROP CONSTRAINT articles_pkey RESTRICT;

DROP VIEW fdv1;

CREATE TEMP VIEW fdv2 AS
SELECT a.id, a.keywords, a.title, aic.category_id, aic.changed
FROM articles AS a JOIN articles_in_category AS aic ON a.id = aic.article_id
WHERE aic.category_id in (14,62,70,53,138)
GROUP BY a.id, aic.category_id, aic.article_id;

ALTER TABLE articles DROP CONSTRAINT articles_pkey RESTRICT;

ALTER TABLE articles_in_category DROP CONSTRAINT articles_in_category_pkey RESTRICT;

DROP VIEW fdv2;

CREATE TEMP VIEW fdv3 AS
SELECT id, keywords, title, body, created
FROM articles
GROUP BY id
UNION
SELECT id, keywords, title, body, created
FROM articles
GROUP BY id;

ALTER TABLE articles DROP CONSTRAINT articles_pkey RESTRICT;

DROP VIEW fdv3;

CREATE TEMP VIEW fdv4 AS
SELECT * FROM articles WHERE title IN (SELECT title FROM articles GROUP BY id);

ALTER TABLE articles DROP CONSTRAINT articles_pkey RESTRICT;

DROP VIEW fdv4;

PREPARE foo AS
  SELECT id, keywords, title, body, created
  FROM articles
  GROUP BY id;

EXECUTE foo;

ALTER TABLE articles DROP CONSTRAINT articles_pkey RESTRICT;

EXECUTE foo;

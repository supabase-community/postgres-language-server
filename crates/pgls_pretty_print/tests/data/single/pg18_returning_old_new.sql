UPDATE products SET price = price * 1.1 WHERE category = 'books' RETURNING WITH (OLD AS o, NEW AS n) o.price AS old_price, n.price AS new_price;

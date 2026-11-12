SELECT * FROM users WHERE id OPERATOR(myschema.=) ANY (SELECT user_id FROM orders);

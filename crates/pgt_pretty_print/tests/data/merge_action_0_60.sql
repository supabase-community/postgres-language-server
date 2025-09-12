MERGE INTO products AS p
USING new_products AS np
ON p.product_id = np.product_id
WHEN MATCHED THEN
    UPDATE SET price = np.price
WHEN NOT MATCHED THEN
    INSERT (product_id, price) VALUES (np.product_id, np.price);
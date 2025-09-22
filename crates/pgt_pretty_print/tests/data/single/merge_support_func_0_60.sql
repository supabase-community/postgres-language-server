MERGE INTO products p
USING new_products np
ON p.product_id = np.product_id
WHEN MATCHED THEN UPDATE SET price = np.price
WHEN NOT MATCHED THEN INSERT VALUES (np.product_id, np.price);
UPDATE indtoasttest SET cnt = cnt +1, f1 = f1 RETURNING substring(indtoasttest::text, 1, 200);

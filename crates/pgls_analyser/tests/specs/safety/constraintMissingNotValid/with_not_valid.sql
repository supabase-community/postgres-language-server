-- Test constraint with NOT VALID (should be safe)
-- expect_no_diagnostics
ALTER TABLE distributors ADD CONSTRAINT distfk FOREIGN KEY (address) REFERENCES addresses (address) NOT VALID;
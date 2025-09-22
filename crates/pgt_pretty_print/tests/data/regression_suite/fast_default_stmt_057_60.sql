ALTER TABLE T ADD COLUMN c_date DATE
                  DEFAULT '2016-06-02'::DATE  + LENGTH(foo(10)),
              ALTER COLUMN c_text SET DEFAULT foo(12);

CREATE TABLE credit_card (
       cid      int references customer(cid),
       cnum     text,
       climit   int
);

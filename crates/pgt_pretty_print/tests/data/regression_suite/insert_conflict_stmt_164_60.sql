create table twoconstraints (f1 int unique, f2 box,
                             exclude using gist(f2 with &&));

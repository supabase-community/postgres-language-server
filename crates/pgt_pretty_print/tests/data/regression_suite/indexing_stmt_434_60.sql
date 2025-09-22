insert into idxpart select a * 2, b || b from idxpart where a between 2^16 and 2^19;

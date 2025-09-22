SELECT b::bit(15), b::bit(15) >> 8 AS bsr8, b::bit(15) << 8 AS bsl8
       FROM BIT_SHIFT_TABLE ;

create type textrange_supp as range (
   subtype = text,
   subtype_opclass = text_pattern_ops
);

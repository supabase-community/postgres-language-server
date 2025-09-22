UPDATE public.tt SET b = b * 2 RETURNING a, b, old.b, new.b, tt.b, public.tt.b;

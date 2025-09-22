create trigger parenttrig after insert on parent
for each row execute procedure f();

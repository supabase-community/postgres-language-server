create trigger parenttrig after insert on child
for each row execute procedure f();

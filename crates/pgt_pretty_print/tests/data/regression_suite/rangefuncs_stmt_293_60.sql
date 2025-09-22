create trigger tnoticetrigger after insert on tt for each row
execute procedure noticetrigger();

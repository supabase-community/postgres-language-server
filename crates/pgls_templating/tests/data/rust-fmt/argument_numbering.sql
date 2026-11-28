-- `{}` takes the next positional argument, `{0}` the first, `{name}` the same one each time
select {}, {}, {0}, {name}, {name:>4}, {:.*}, {};

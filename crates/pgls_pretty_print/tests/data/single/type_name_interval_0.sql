CREATE TABLE interval_samples (
  plain_interval INTERVAL,
  precision_only INTERVAL(3),
  year_only INTERVAL YEAR,
  month_only INTERVAL MONTH,
  year_to_month INTERVAL YEAR TO MONTH,
  day_only INTERVAL DAY,
  day_to_hour INTERVAL DAY TO HOUR,
  day_to_minute INTERVAL DAY TO MINUTE,
  day_to_second INTERVAL DAY TO SECOND(4),
  hour_only INTERVAL HOUR,
  hour_to_minute INTERVAL HOUR TO MINUTE,
  hour_to_second INTERVAL HOUR TO SECOND(2),
  minute_only INTERVAL MINUTE,
  minute_to_second INTERVAL MINUTE TO SECOND,
  second_only INTERVAL SECOND,
  second_precision INTERVAL SECOND(6)
);

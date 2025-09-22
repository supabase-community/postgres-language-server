SELECT width_bucket(now(),
                    array['yesterday', 'today', 'tomorrow']::timestamptz[]);

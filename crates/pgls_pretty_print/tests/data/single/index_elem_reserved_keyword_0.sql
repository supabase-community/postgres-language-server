CREATE UNIQUE INDEX index_user_emails_on_user_id_and_primary ON public.user_emails USING btree (user_id, "primary") WHERE "primary";

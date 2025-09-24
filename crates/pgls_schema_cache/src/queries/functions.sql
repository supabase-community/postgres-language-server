with functions as (
  select
    oid,
    proname,
    prosrc,
    prorettype,
    proretset,
    provolatile,
    prosecdef,
    prolang,
    pronamespace,
    proconfig,
    prokind,
    -- proargmodes is null when all arg modes are IN
    coalesce(
      p.proargmodes,
      array_fill(
        'i' :: text,
        array [cardinality(coalesce(p.proallargtypes, p.proargtypes))]
      )
    ) as arg_modes,
    -- proargnames is null when all args are unnamed
    coalesce(
      p.proargnames,
      array_fill(
        '' :: text,
        array [cardinality(coalesce(p.proallargtypes, p.proargtypes))]
      )
    ) as arg_names,
    -- proallargtypes is null when all arg modes are IN
    coalesce(p.proallargtypes, string_to_array(proargtypes::text, ' ')::int[]) as arg_types,
    array_cat(
      array_fill(false, array [pronargs - pronargdefaults]),
      array_fill(true, array [pronargdefaults])
    ) as arg_has_defaults
  from
    pg_proc as p
)
select
  f.oid :: int8 as "id!",
  n.nspname as "schema!",
  f.proname as "name!",
  l.lanname as "language!",
  f.prokind as "kind!",
  case
    when l.lanname = 'internal' then null
    else f.prosrc
  end as body,
  case
    when l.lanname = 'internal' then null
    else pg_get_functiondef(f.oid)
  end as definition,
  coalesce(f_args.args, '[]') as args,
  nullif(pg_get_function_arguments(f.oid), '') as argument_types,
  nullif(pg_get_function_identity_arguments(f.oid), '') as identity_argument_types,
  f.prorettype :: int8 as return_type_id,
  pg_get_function_result(f.oid) as return_type,
  nullif(rt.typrelid :: int8, 0) as return_type_relation_id,
  f.proretset as "is_set_returning_function!",
  case
    when f.provolatile = 'i' then 'IMMUTABLE'
    when f.provolatile = 's' then 'STABLE'
    when f.provolatile = 'v' then 'VOLATILE'
  end as behavior,
  f.prosecdef as "security_definer!"
from
  functions f
  left join pg_namespace n on f.pronamespace = n.oid
  left join pg_language l on f.prolang = l.oid
  left join pg_type rt on rt.oid = f.prorettype
  left join (
    select
      oid,
      jsonb_object_agg(param, value) filter (
        where
          param is not null
      ) as config_params
    from
      (
        select
          oid,
          (string_to_array(unnest(proconfig), '=')) [1] as param,
          (string_to_array(unnest(proconfig), '=')) [2] as value
        from
          functions
      ) as t
    group by
      oid
  ) f_config on f_config.oid = f.oid
  left join (
    select
      oid,
      jsonb_agg(
        jsonb_build_object(
          'mode',
          t2.mode,
          'name',
          name,
          'type_id',
          type_id,
          'has_default',
          has_default
        )
      ) as args
    from
      (
        select
          oid,
          arg_modes[i] as mode,
          arg_names[i] as name,
          arg_types[i] :: int8 as type_id,
          arg_has_defaults[i] as has_default
        from
          functions,
          pg_catalog.generate_subscripts(arg_names, 1) as i
      ) as t1,
      lateral (
        select
          case
            when t1.mode = 'i' then 'in'
            when t1.mode = 'o' then 'out'
            when t1.mode = 'b' then 'inout'
            when t1.mode = 'v' then 'variadic'
            else 'table'
          end as mode
      ) as t2
    group by
      t1.oid
  ) f_args on f_args.oid = f.oid;
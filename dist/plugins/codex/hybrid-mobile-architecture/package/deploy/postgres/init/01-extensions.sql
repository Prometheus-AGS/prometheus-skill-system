CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS pg_net;
CREATE EXTENSION IF NOT EXISTS pg_cron;
CREATE EXTENSION IF NOT EXISTS flint_llm;
CREATE EXTENSION IF NOT EXISTS flint_vault;
CREATE EXTENSION IF NOT EXISTS flint_meta;
CREATE EXTENSION IF NOT EXISTS flint_auth;
CREATE EXTENSION IF NOT EXISTS flint_hooks;

DO $$
DECLARE
  required text[] := ARRAY['vector','pgcrypto','pg_net','pg_cron','flint_llm','flint_vault','flint_meta','flint_auth','flint_hooks'];
  missing text[];
BEGIN
  SELECT array_agg(name) INTO missing
  FROM unnest(required) AS name
  WHERE NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = name);
  IF missing IS NOT NULL THEN
    RAISE EXCEPTION 'missing required Prometheus extensions: %', missing;
  END IF;
END $$;

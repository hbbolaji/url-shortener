-- Add migration script here
CREATE TABLE urls(
  id SERIAL PRIMARY KEY,
  long_url TEXT NOT NULL,
  crated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
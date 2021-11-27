CREATE TABLE actions (
  id BIGSERIAL PRIMARY KEY,
  guild_id BIGINT NOT NULL,
  user_id BIGINT,
  regex TEXT,
  reactions char(1)[] NOT NULL
)

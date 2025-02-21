CREATE TABLE IF NOT EXISTS reactions (
  slug TEXT NOT NULL,
  emoji TEXT NOT NULL,
  uid TEXT NOT NULL,
  PRIMARY KEY(slug, emoji, uid)
);

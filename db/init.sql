PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS "clients" (
  "id"      INTEGER PRIMARY KEY AUTOINCREMENT,
  "limit"   INTEGER NOT NULL,
  "balance" INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS "transactions" (
  "id"          INTEGER PRIMARY KEY AUTOINCREMENT,
  "kind"        CHAR(1)     NOT NULL,
  "value"      INTEGER     NOT NULL,
  "description" VARCHAR(10) NOT NULL,
  "client_id"   INTEGER     NOT NULL,
  "created_at"  TIMESTAMP   NOT NULL,

  FOREIGN KEY ("client_id") REFERENCES "clients" ("id")
);

INSERT INTO "clients" ("id", "limit") VALUES (1, 100000);
INSERT INTO "clients" ("id", "limit") VALUES (2, 80000);
INSERT INTO "clients" ("id", "limit") VALUES (3, 1000000);
INSERT INTO "clients" ("id", "limit") VALUES (4, 10000000);
INSERT INTO "clients" ("id", "limit") VALUES (5, 500000);

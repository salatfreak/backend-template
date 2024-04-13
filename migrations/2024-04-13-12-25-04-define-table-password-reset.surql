-- password reset table
DEFINE TABLE password_reset SCHEMAFULL;

DEFINE FIELD token ON password_reset
  TYPE string
  DEFAULT rand::string();

DEFINE FIELD user ON password_reset
  TYPE record<user>;

DEFINE FIELD expires ON password_reset
  TYPE datetime;

DEFINE INDEX token ON password_reset
  COLUMNS token
  UNIQUE;

DEFINE INDEX user ON password_reset
  COLUMNS user
  UNIQUE;

-- delete password resets when deleting user
DEFINE EVENT delete_password_resets ON user
WHEN $event = "DELETE"
THEN (
  DELETE password_reset WHERE user = $before.id
);

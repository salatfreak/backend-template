DEFINE TABLE login SCHEMAFULL;

DEFINE FIELD token ON login
  TYPE string
  DEFAULT rand::string();

DEFINE FIELD user ON login
  TYPE record<user>;

DEFINE INDEX token ON login
  COLUMNS token
  UNIQUE;

-- delete logins when deleting user
DEFINE EVENT delete_logins ON user
WHEN $event = "DELETE"
THEN (
  DELETE login WHERE user = $before.id
);

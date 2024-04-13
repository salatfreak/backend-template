-- registration table with password hashing
DEFINE TABLE registration SCHEMAFULL;

DEFINE FIELD token ON registration
  TYPE string
  DEFAULT rand::string();

DEFINE FIELD data ON registration
  FLEXIBLE
  TYPE object;

DEFINE FIELD data.password ON registration
  TYPE string
  VALUE crypto::argon2::generate($value);

DEFINE FIELD data.email ON registration
  TYPE string
  VALUE string::lowercase($value)
  ASSERT string::is::email($value);

DEFINE FIELD expires ON registration
  TYPE datetime;

DEFINE INDEX token ON registration
  COLUMNS token
  UNIQUE;

-- remove password hashing on user table
REMOVE FIELD password ON user;
DEFINE FIELD password ON user
  TYPE string
  ASSERT $value != "";

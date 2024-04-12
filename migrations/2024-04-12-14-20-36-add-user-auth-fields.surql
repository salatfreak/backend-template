DEFINE FIELD email ON user
  TYPE string
  VALUE string::lowercase($value)
  ASSERT string::is::email($value);

DEFINE FIELD password ON user
  TYPE string
  VALUE crypto::argon2::generate($value);

DEFINE INDEX email ON user
  COLUMNS email
  UNIQUE;

DEFINE TABLE user SCHEMAFULL;

DEFINE FIELD name ON user
  TYPE string
  VALUE string::trim($value)
  ASSERT string::len($value) >= 2;

DEFINE FIELD role ON user
  TYPE string
  DEFAULT "user"
  ASSERT $value in ["user", "admin", "owner"];

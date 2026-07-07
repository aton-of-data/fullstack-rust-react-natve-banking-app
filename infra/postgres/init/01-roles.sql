-- Create roles and database permissions
CREATE ROLE ficus_migrator WITH LOGIN PASSWORD 'ficus_migrator_password';
CREATE ROLE ficus_app WITH LOGIN PASSWORD 'ficus_app_password';

GRANT CONNECT ON DATABASE ficus TO ficus_migrator, ficus_app;
GRANT CREATE ON DATABASE ficus TO ficus_migrator;

\c ficus

GRANT USAGE ON SCHEMA public TO ficus_migrator, ficus_app;
GRANT CREATE ON SCHEMA public TO ficus_migrator;

-- App role: read/write on application tables (granted after migrations)
ALTER DEFAULT PRIVILEGES FOR ROLE ficus_migrator IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE ON TABLES TO ficus_app;

ALTER DEFAULT PRIVILEGES FOR ROLE ficus_migrator IN SCHEMA public
  GRANT USAGE, SELECT ON SEQUENCES TO ficus_app;

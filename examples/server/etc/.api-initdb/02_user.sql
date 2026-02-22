\c api_database;

CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO user_api;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO user_api;

INSERT INTO users (id, name, email) VALUES (1, 'John Doe', 'john@example.com');
INSERT INTO users (id, name, email) VALUES (2, 'Jane Doe', 'jane@example.com');
INSERT INTO users (id, name, email) VALUES (3, 'Bob Smith', 'bob@example.com');
INSERT INTO users (id, name, email) VALUES (4, 'Alice Johnson', 'alice@example.com');
INSERT INTO users (id, name, email) VALUES (5, 'Charlie Brown', 'charlie@example.com');
INSERT INTO users (id, name, email) VALUES (6, 'Diana Miller', 'diana@example.com');

ALTER SEQUENCE users_id_seq RESTART WITH 7;

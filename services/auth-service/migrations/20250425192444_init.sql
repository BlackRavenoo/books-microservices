CREATE TABLE users (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name VARCHAR(127) NOT NULL,
    email VARCHAR(127) NOT NULL UNIQUE,
    password_hash VARCHAR(256) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE clients (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE client_redirect_uris (
    id SERIAL PRIMARY KEY,
    client_id VARCHAR(255) NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    redirect_uri VARCHAR(2048) NOT NULL,
    UNIQUE (client_id, redirect_uri)
);

CREATE TABLE roles (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name VARCHAR(50) NOT NULL UNIQUE
);

CREATE TABLE user_roles (
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    role_id INT REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

INSERT INTO clients (id, name) VALUES 
    ('book-app', 'Book Reading App');

INSERT INTO client_redirect_uris (client_id, redirect_uri) VALUES
    ('book-app', 'http://127.0.0.1/callback'),
    ('book-app', 'http://localhost/callback');

INSERT INTO roles (name) VALUES 
    ('admin');
PRAGMA foreign_keys = ON;

CREATE TABLE config (
    key TEXT NOT NULL UNIQUE,
    value TEXT NOT NULL
);

INSERT INTO config (key, value) VALUES 
("user_mode", json_quote("SingleUserManualLogin")),
("terminal_editor", json_quote("/bin/vim"));

CREATE TABLE users (
    user_id INTEGER PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    last_visited_zettel INTEGER REFERENCES zettel(zettel_id)
);

CREATE UNIQUE INDEX idx_users_username ON users(username);

CREATE TABLE zettel (
    zettel_id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(user_id),
    path TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    created_on DATETIME NOT NULL,
    last_modified_on DATETIME NOT NULL
);

CREATE UNIQUE INDEX idx_zettel_user_path ON zettel(user_id, path);

CREATE TABLE zettel_attachment (
    zettel_attachment_id INTEGER PRIMARY KEY NOT NULL,
    zettel_id INTEGER NOT NULL REFERENCES zettel(zettel_id),
    path TEXT NOT NULL
);

CREATE TABLE zettel_link (
    source_zettel_id INTEGER NOT NULL REFERENCES zettel(zettel_id),
    destination_zettel_id INTEGER NOT NULL REFERENCES zettel(zettel_id)
);

CREATE TABLE zettel_history (
    zettel_id INTEGER NOT NULL REFERENCES zettel(zettel_id),
    time DATETIME NOT NULL,
    patch TEXT NOT NULL
);

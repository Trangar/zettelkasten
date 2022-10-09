PRAGMA foreign_keys = ON;

CREATE TABLE users (
    user_id INTEGER PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_users_username ON users(username);

CREATE TABLE page (
    page_id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(user_id),
    path TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL
);

CREATE TABLE page_attachment (
    page_attachment_id INTEGER PRIMARY KEY NOT NULL,
    page_id INTEGER NOT NULL REFERENCES page(page_id),
    path TEXT NOT NULL
);

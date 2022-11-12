CREATE TABLE config (
    key TEXT NOT NULL UNIQUE,
    value TEXT NOT NULL
);

INSERT INTO config (key, value) VALUES 
('user_mode', '"SingleUserManualLogin"'),
('terminal_editor', '"/bin/vim"');

CREATE TABLE users (
    user_id BIGSERIAL PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    last_visited_zettel BIGINT NULL
);

CREATE UNIQUE INDEX idx_users_username ON users(username);

CREATE TABLE zettel (
    zettel_id BIGSERIAL PRIMARY KEY NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users(user_id),
    path TEXT NOT NULL,
    body TEXT NOT NULL,
    created_on TIMESTAMP WITH TIME ZONE NOT NULL,
    last_modified_on TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE UNIQUE INDEX idx_zettel_user_path ON zettel(user_id, path);
ALTER TABLE users ADD CONSTRAINT fk_users_last_visited_zettel FOREIGN KEY (last_visited_zettel) REFERENCES zettel (zettel_id);

CREATE TABLE zettel_attachment (
    zettel_attachment_id BIGSERIAL PRIMARY KEY NOT NULL,
    zettel_id BIGINT NOT NULL REFERENCES zettel(zettel_id),
    path TEXT NOT NULL
);

CREATE TABLE zettel_link (
    source_zettel_id BIGSERIAL NOT NULL REFERENCES zettel(zettel_id),
    destination_zettel_id BIGINT NOT NULL REFERENCES zettel(zettel_id)
);

CREATE TABLE zettel_history (
    zettel_id BIGSERIAL NOT NULL REFERENCES zettel(zettel_id),
    time TIMESTAMP WITH TIME ZONE NOT NULL,
    patch TEXT NOT NULL
);


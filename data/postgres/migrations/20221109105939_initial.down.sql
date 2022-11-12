ALTER TABLE users DROP CONSTRAINT fk_users_last_visited_zettel;

DROP TABLE zettel_history;
DROP TABLE zettel_link;
DROP TABLE zettel_attachment;
DROP TABLE zettel;
DROP TABLE users;
DROP TABLE config;


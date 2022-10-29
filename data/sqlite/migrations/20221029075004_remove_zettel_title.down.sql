-- Add down migration script here

ALTER TABLE zettel ADD COLUMN title TEXT;
UPDATE zettel SET title = "";

-- we need to edit the column `path` to be COLLATE NOCASE
-- however sqlite does not have this
-- we need to:
-- - create a new column
-- - copy over the path 
-- - delete index `idx_zettel_user_path`
-- - delete the old path column
-- - create the path we need
-- - copy over the old path
-- - re-create index `idx_zettel_user_path`
ALTER TABLE zettel ADD COLUMN path_tmp TEXT;
UPDATE zettel SET path_tmp = path;
DROP INDEX idx_zettel_user_path;
ALTER TABLE zettel DROP COLUMN path;
ALTER TABLE zettel ADD COLUMN path TEXT NOT NULL DEFAULT('') COLLATE NOCASE;
UPDATE zettel SET path = path_tmp;
ALTER TABLE zettel DROP COLUMN path_tmp;
CREATE UNIQUE INDEX idx_zettel_user_path ON zettel(user_id, path);

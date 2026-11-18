-- reports are exported as csv only, so the format column is no longer needed
ALTER TABLE reports DROP COLUMN format;

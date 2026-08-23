-- Logo/photo du commerce, uploadée par le marchand vers MinIO/S3 (même
-- stockage que les photos de panier). Nullable : un marchand reste
-- utilisable sans logo.
ALTER TABLE marchands
    ADD COLUMN logo_url TEXT;

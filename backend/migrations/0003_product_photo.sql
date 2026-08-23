-- Photo d'un panier, uploadée par le marchand vers MinIO/S3 (cf. backend
-- infrastructure/storage). Nullable : un panier reste publiable sans photo.
ALTER TABLE produits
    ADD COLUMN photo_url TEXT;

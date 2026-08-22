-- Position du marchand (captée via navigator.geolocation au moment de
-- l'inscription, cf. VISION.md §8) - nullable : un marchand qui refuse le
-- partage de position à l'inscription reste utilisable, simplement sans
-- distance affichée côté consommateur.

ALTER TABLE marchands
    ADD COLUMN latitude DOUBLE PRECISION CHECK (latitude IS NULL OR (latitude >= -90 AND latitude <= 90)),
    ADD COLUMN longitude DOUBLE PRECISION CHECK (longitude IS NULL OR (longitude >= -180 AND longitude <= 180));

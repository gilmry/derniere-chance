-- Réserver plusieurs paniers d'un même commerce en une fois, retirés avec
-- un seul code : une ligne par panier, toutes avec le même code, et une
-- quantité par ligne (2 paniers identiques = une ligne, quantite = 2).
--
-- Le code n'est donc plus unique par ligne, mais par réservation : l'unicité
-- est vérifiée à l'insertion (verrou sur le code, puis contrôle qu'il est
-- libre, voir `PostgresReservationRepository::create_group`). L'index
-- (code, produit_id) empêche seulement un même panier en double sous un code.

ALTER TABLE reservations
    ADD COLUMN quantite INTEGER NOT NULL DEFAULT 1 CHECK (quantite > 0);

ALTER TABLE reservations DROP CONSTRAINT IF EXISTS reservations_code_key;

CREATE UNIQUE INDEX reservations_code_produit ON reservations (code, produit_id);
CREATE INDEX idx_reservations_code ON reservations (code);

-- DernièreChance: initial schema (marchands, produits, consommateurs, abonnements, reservations, notifications)

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE product_status AS ENUM ('publie', 'ecoule', 'expire');
CREATE TYPE reservation_status AS ENUM ('reservee', 'recuperee', 'expiree');
CREATE TYPE notification_status AS ENUM ('envoyee', 'echouee');

CREATE TABLE marchands (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nom TEXT NOT NULL,
    adresse TEXT NOT NULL,
    categorie TEXT NOT NULL,
    note NUMERIC(2, 1) CHECK (note IS NULL OR (note >= 0 AND note <= 5)),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE consommateurs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE produits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    marchand_id UUID NOT NULL REFERENCES marchands(id) ON DELETE CASCADE,
    nom TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    prix_initial NUMERIC(10, 2) NOT NULL CHECK (prix_initial >= 0),
    prix_demarque NUMERIC(10, 2) NOT NULL CHECK (prix_demarque >= 0),
    quantite INTEGER NOT NULL CHECK (quantite >= 0),
    retrait_debut TIMESTAMPTZ NOT NULL,
    retrait_fin TIMESTAMPTZ NOT NULL,
    statut product_status NOT NULL DEFAULT 'publie',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT prix_demarque_inferieur CHECK (prix_demarque <= prix_initial),
    CONSTRAINT retrait_coherent CHECK (retrait_fin > retrait_debut)
);

CREATE INDEX idx_produits_marchand ON produits (marchand_id);
CREATE INDEX idx_produits_statut ON produits (statut);

CREATE TABLE abonnements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    consommateur_id UUID NOT NULL REFERENCES consommateurs(id) ON DELETE CASCADE,
    marchand_id UUID NOT NULL REFERENCES marchands(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (consommateur_id, marchand_id)
);

CREATE INDEX idx_abonnements_marchand ON abonnements (marchand_id);

CREATE TABLE reservations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    produit_id UUID NOT NULL REFERENCES produits(id) ON DELETE CASCADE,
    consommateur_id UUID NOT NULL REFERENCES consommateurs(id) ON DELETE CASCADE,
    code TEXT NOT NULL UNIQUE,
    statut reservation_status NOT NULL DEFAULT 'reservee',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_reservations_consommateur ON reservations (consommateur_id);
CREATE INDEX idx_reservations_produit ON reservations (produit_id);

CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    produit_id UUID NOT NULL REFERENCES produits(id) ON DELETE CASCADE,
    consommateur_id UUID NOT NULL REFERENCES consommateurs(id) ON DELETE CASCADE,
    statut notification_status NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_notifications_consommateur ON notifications (consommateur_id);

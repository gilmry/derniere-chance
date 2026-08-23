-- Backoffice admin léger (un seul compte, pas d'auto-inscription publique -
-- créé au démarrage depuis ADMIN_EMAIL/ADMIN_PASSWORD si aucun n'existe,
-- cf. infrastructure/bootstrap.rs, même pattern qu'Elevia).
CREATE TABLE admins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

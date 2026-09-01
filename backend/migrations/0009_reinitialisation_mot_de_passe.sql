-- Réinitialisation de mot de passe par email, pour les deux principaux
-- auto-inscrits (consommateurs et marchands).
--
-- Une seule table à deux colonnes exclusives, comme consentements_beta : la
-- mécanique à tenir est la même pour les deux (un secret à usage unique, une
-- expiration), et deux schémas jumeaux devraient évoluer en double.
--
-- Le jeton lui-même n'est jamais stocké. Seule son empreinte SHA-256 l'est,
-- au même titre qu'un mot de passe ou qu'un refresh token OAuth : une fuite
-- de la base ne doit pas suffire à prendre la main sur des comptes.

CREATE TABLE reinitialisations_mot_de_passe (
    token_hash TEXT PRIMARY KEY,
    consommateur_id UUID REFERENCES consommateurs(id) ON DELETE CASCADE,
    marchand_id UUID REFERENCES marchands(id) ON DELETE CASCADE,
    expire_le TIMESTAMPTZ NOT NULL,
    -- Horodaté à la consommation : un lien de réinitialisation ne vaut
    -- qu'une fois, sans quoi un email qui traîne dans une boîte reste une
    -- clé du compte.
    utilise_le TIMESTAMPTZ,
    cree_le TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT reinitialisation_un_seul_sujet
        CHECK ((consommateur_id IS NULL) <> (marchand_id IS NULL))
);

CREATE INDEX idx_reinitialisations_consommateur
    ON reinitialisations_mot_de_passe (consommateur_id);
CREATE INDEX idx_reinitialisations_marchand
    ON reinitialisations_mot_de_passe (marchand_id);
-- Sert au ménage des jetons périmés, fait à chaque nouvelle demande.
CREATE INDEX idx_reinitialisations_expire ON reinitialisations_mot_de_passe (expire_le);

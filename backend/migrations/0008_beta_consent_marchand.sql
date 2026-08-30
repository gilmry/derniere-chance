-- Étend le consentement bêta aux marchands.
--
-- Un marchand confie davantage qu'un consommateur : nom du commerce, adresse
-- postale et, s'il l'accepte, sa position GPS - toutes publiées sur la carte
-- publique. Pour un commerçant en personne physique, ce sont des données
-- personnelles, et elles méritent le même acte de consentement explicite.
--
-- Une seule table plutôt que deux : marchands et consommateurs sont deux
-- principaux distincts, mais la preuve à produire est la même (qui, quand,
-- quelle version). Deux colonnes exclusives l'une de l'autre valent mieux
-- qu'un schéma dupliqué qu'il faudrait faire évoluer en double.

ALTER TABLE consentements_beta
    ADD COLUMN marchand_id UUID REFERENCES marchands(id) ON DELETE CASCADE;

ALTER TABLE consentements_beta
    ALTER COLUMN consommateur_id DROP NOT NULL;

ALTER TABLE consentements_beta
    ADD CONSTRAINT consentement_un_seul_sujet
    CHECK ((consommateur_id IS NULL) <> (marchand_id IS NULL));

-- Pendant marchand de idx_consentements_beta_actif. Les deux index partiels
-- coexistent sans se gêner : une ligne consommateur a marchand_id NULL, et
-- les NULL ne s'affrontent jamais dans un index unique.
CREATE UNIQUE INDEX idx_consentements_beta_actif_marchand
    ON consentements_beta (marchand_id)
    WHERE retire_le IS NULL;

CREATE INDEX idx_consentements_beta_marchand
    ON consentements_beta (marchand_id);

-- Marque un compte marchand dont le consentement a été retiré : nom,
-- adresse, position et logo remplacés par des valeurs neutres.
ALTER TABLE marchands ADD COLUMN anonymise_le TIMESTAMPTZ;

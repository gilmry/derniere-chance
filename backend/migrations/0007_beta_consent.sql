-- Programme bêta : traçabilité du consentement RGPD.
--
-- L'article 7 §1 impose au responsable de traitement de pouvoir démontrer
-- que la personne a consenti, et à quoi : on garde donc une ligne par acte
-- de consentement, avec la version du texte affiché au moment de
-- l'acceptation. Rien n'est écrasé, seul le retrait est horodaté en place.
--
-- Le retrait déclenche une anonymisation du compte, pas un DELETE :
-- supprimer la ligne consommateurs ferait disparaître par cascade la preuve
-- de consentement qu'on doit justement conserver, ainsi que les
-- réservations déjà honorées par les commerçants (leurs statistiques).
-- Une fois anonymisée, la ligne ne contient plus de donnée personnelle,
-- donc la conserver reste conforme.

CREATE TABLE consentements_beta (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    consommateur_id UUID NOT NULL REFERENCES consommateurs(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    accepte_le TIMESTAMPTZ NOT NULL DEFAULT now(),
    retire_le TIMESTAMPTZ
);

-- Au plus un consentement actif (non retiré) par consommateur : rend un
-- double POST /consommateurs/moi/consentement inoffensif côté base plutôt
-- que de compter sur le seul contrôle applicatif.
CREATE UNIQUE INDEX idx_consentements_beta_actif
    ON consentements_beta (consommateur_id)
    WHERE retire_le IS NULL;

CREATE INDEX idx_consentements_beta_consommateur
    ON consentements_beta (consommateur_id);

-- Marque un compte dont le consentement a été retiré : email et mot de
-- passe y ont été remplacés par des valeurs non identifiantes et la
-- connexion est devenue impossible.
ALTER TABLE consommateurs ADD COLUMN anonymise_le TIMESTAMPTZ;

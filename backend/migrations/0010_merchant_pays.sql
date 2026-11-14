-- Pays du commerce (code ISO 3166-1 alpha-2). Il borne la recherche du
-- géocodeur, qui calcule désormais la position à partir de l'adresse saisie
-- au lieu de la position du navigateur au moment de l'inscription.
--
-- Les commerces déjà inscrits passent en France ; leur position actuelle
-- (celle du navigateur) est gardée jusqu'à leur prochaine modification de
-- profil, qui la recalcule depuis l'adresse.

ALTER TABLE marchands
    ADD COLUMN pays CHAR(2) NOT NULL DEFAULT 'FR' CHECK (pays ~ '^[A-Z]{2}$');

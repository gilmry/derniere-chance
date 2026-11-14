-- Adresse que le géocodeur ne trouve pas (rue récente, base OpenStreetMap pas
-- à jour), acceptée parce que le commerçant a coché une case pour confirmer
-- que son commerce existe bien à cette adresse. On garde la date de cette
-- confirmation : c'est l'engagement du commerçant, comme pour le consentement.
-- NULL = adresse trouvée par le géocodeur. Remis à NULL dès qu'une nouvelle
-- adresse est trouvée.

ALTER TABLE marchands
    ADD COLUMN adresse_confirmee_le TIMESTAMPTZ;

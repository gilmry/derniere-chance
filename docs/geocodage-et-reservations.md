# Résumé des modifications — branche `feat/geocodage-europeen`

## Commerçants

- Choix du pays parmi 32 pays européens ; la position du commerce vient de
  son adresse, et non plus du navigateur.
- Suggestions d'adresses pendant la frappe (OpenStreetMap).
- Vérification que le commerce existe à cette adresse ; sinon, message clair
  et case pour l'ajouter manuellement.
- L'adresse complète trouvée remplace ce qui a été tapé.
- Tableau de bord : adresse en entier, statut « trouvé » ou « ajouté
  manuellement », compteurs à jour.

## Clients

- Le fil liste les commerces (avec leur logo) ; un clic montre leurs paniers.
- Réservation de plusieurs paniers d'un même commerce en une fois, avec un
  seul code de retrait.

## Admin

- Pays et alerte « commerce non vérifié » affichés ; listes et compteurs
  rechargés après chaque action.

## Corrections

- Page commerce en erreur pour un client connecté.
- « internal error » sur l'accord bêta quand le compte avait été supprimé.

## Sécurité

- Limite de demandes par adresse IP sur l'inscription et les suggestions.
- Politique de confidentialité et registre RGPD : OpenStreetMap déclaré.
- 10 paniers au plus en attente de retrait par client et par commerce.
- Redirections limitées aux pages du site.

## Technique

- Migrations `0010` à `0012` (pays, ajout manuel, quantité par réservation).
- Nouvelles variables : `GEOCODEUR`, `NOMINATIM_URL`, `PHOTON_URL`,
  `LIMITE_INSCRIPTIONS_PAR_HEURE`, `LIMITE_SUGGESTIONS_PAR_MINUTE`.
- 98 tests backend réussis, `astro check` sans erreur.

## À décider

- Nouvelle version de la politique de confidentialité (redemande l'accord de
  tous) ?
- Serveurs OpenStreetMap publics ou auto-hébergés ?
- Codes de retrait plus longs (10 000 possibles aujourd'hui) ?

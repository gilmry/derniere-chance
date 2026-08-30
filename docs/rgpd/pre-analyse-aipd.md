# Pré-analyse AIPD — programme bêta DernièreChance

**Conclusion : aucune analyse d'impact n'est requise.**

Ce document existe pour le prouver. L'article 35 n'impose une AIPD que si le
traitement est « susceptible d'engendrer un risque élevé » ; mais l'article 5.2
impose de pouvoir **démontrer** qu'on s'est posé la question et comment on y a
répondu. Affirmer « il n'y a pas de risque élevé » ne vaut rien en contrôle ;
un screening daté, critère par critère, vaut quelque chose.

- **Traitement examiné** : programme bêta DernièreChance (voir
  [`registre-traitements.md`](./registre-traitements.md))
- **Responsable du traitement** : Gilles Maury, Bruxelles — BE0670.778.061
- **Autorité compétente** : APD (chef de file, art. 56) ; CNIL autorité
  concernée pour les testeurs français
- **Date du screening** : 30 août 2026
- **Version du texte de consentement en vigueur à cette date** : `2026-08-30`

---

## 1. Cas imposés par l'article 35.3

| Cas | Applicable ? |
|---|---|
| Évaluation systématique et approfondie fondée sur un traitement automatisé, y compris le profilage, avec effets juridiques ou significatifs | **Non** — aucun profilage, aucune décision automatisée |
| Traitement à grande échelle de données sensibles (art. 9) ou pénales (art. 10) | **Non** — aucune donnée de ces catégories n'est collectée |
| Surveillance systématique à grande échelle d'une zone accessible au public | **Non** — voir §3 sur la géolocalisation |

## 2. Liste de l'autorité de contrôle (art. 35.4)

L'établissement unique du responsable étant en Belgique, la liste opposable
est celle de l'**APD** (décision n° 01/2019), et non celle de la CNIL.

Aucune des catégories qu'elle vise ne correspond : ni données biométriques ou
génétiques, ni données de santé, ni évaluation de solvabilité ou de
comportement, ni traitement à grande échelle de données très personnelles, ni
collecte auprès de tiers en vue d'une décision, ni surveillance de zone
accessible au public.

*Réserve* : cette conclusion s'appuie sur la lecture de synthèses de la
décision 01/2019, non sur son texte intégral. À confirmer lors de la première
relecture juridique (voir §6).

## 3. Les neuf critères du CEPD (WP248 rév.01)

Deux critères remplis font présumer un risque élevé. Le traitement en remplit
**zéro**.

| # | Critère | Réponse | Justification |
|---|---|---|---|
| 1 | Évaluation ou notation (scoring) | Non | Aucun score, aucun classement de testeurs |
| 2 | Décision automatisée à effet juridique ou significatif | Non | Aucune décision automatisée (art. 22 sans objet) |
| 3 | Surveillance systématique | Non | Voir ci-dessous |
| 4 | Données sensibles ou hautement personnelles | Non | Ni art. 9, ni art. 10, ni données financières |
| 5 | Traitement à grande échelle | Non | Quelques dizaines de testeurs, une seule ville pilote, durée bornée au bêta |
| 6 | Croisement ou combinaison d'ensembles de données | Non | Aucune source externe, aucun enrichissement |
| 7 | Personnes vulnérables | Non | Ni mineurs, ni patients, ni salariés en lien de subordination |
| 8 | Usage innovant de technologies | Non | Application web classique, aucune biométrie, aucune IA décisionnelle |
| 9 | Le traitement empêche d'exercer un droit ou de bénéficier d'un service | Non | Le refus de consentir n'ouvre ni ne ferme aucun droit hors du bêta lui-même |

### Le point sensible : la géolocalisation (critères 3 et 5)

C'est le seul endroit où le traitement s'approche d'un déclencheur, et
plusieurs guides citent « la collecte de données de géolocalisation » parmi
les cas d'AIPD. Il n'est pas rempli ici, pour deux raisons distinctes :

- **Côté consommateur, rien n'est conservé.** La position, si le navigateur
  l'autorise, est transmise en paramètre de requête, sert au calcul d'une
  distance, et disparaît avec la requête. Aucune écriture en base, donc aucun
  historique, donc aucun suivi de déplacement possible.
- **Côté marchand, c'est une adresse, pas une trajectoire.** La position est
  celle d'un **point de vente fixe**, saisie une fois à l'inscription,
  facultative, et destinée à être publiée sur la carte. Ce que vise le critère
  de surveillance systématique, c'est le suivi dans le temps des déplacements
  de personnes. Un commerce qui publie où il se trouve n'en relève pas.

## 4. Conclusion

Zéro critère sur neuf, aucun cas de l'article 35.3, aucune entrée de la liste
APD. **Pas d'AIPD requise**, et *a fortiori* pas de consultation préalable de
l'APD au titre de l'article 36, celle-ci ne s'imposant qu'au terme d'une AIPD
concluant à un risque résiduel élevé.

Cette conclusion vaut pour le périmètre décrit dans le registre à la date
ci-dessus. Elle n'est pas acquise pour la suite.

## 5. Ce qui obligerait à refaire ce screening

À réexaminer **avant** de livrer l'une de ces évolutions, pas après :

| Évolution | Critère qui basculerait |
|---|---|
| Notification de proximité (« préviens-moi quand je passe près d'un commerce ») | 3 — géolocalisation continue = surveillance systématique |
| Filtres allergènes ou régime (sans gluten, halal, végétarien) | 4 — données de santé ou de conviction religieuse (art. 9) |
| Ouverture au grand public, ou changement d'échelle du nombre de testeurs | 5 — grande échelle |
| Recommandation personnalisée, score de fiabilité client ou marchand | 1, et 2 si une décision en découle |
| Encaissement en ligne, collecte de données de paiement | 4 |
| Ouverture aux mineurs | 7 |
| Recours à un sous-traitant hors UE | Transferts (chap. V), à traiter séparément |

Les deux premières lignes sont des évolutions naturelles de ce produit : elles
méritent d'être signalées ici plutôt que découvertes en cours de
développement.

## 6. Limites

Ce screening est un travail d'ingénierie documenté, pas un avis juridique. Il
est proportionné à un bêta fermé. Avant toute mise en production chez une
organisation tierce, une relecture par un juriste spécialisé est recommandée,
ne serait-ce que pour confirmer le §2 sur pièces.

---

## Journal des révisions

| Date | Modification |
|---|---|
| 2026-08-30 | Création. Screening initial du programme bêta : zéro critère CEPD sur neuf, pas d'AIPD requise. |

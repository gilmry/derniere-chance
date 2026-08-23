# Serveur MCP marchand (OAuth 2.1 + PKCE maison)

DernièreChance expose un endpoint `POST /mcp` (JSON-RPC 2.0, transport MCP
"Streamable HTTP") permettant à un marchand de connecter un client MCP
(Claude Code, Claude Desktop, claude.ai) directement à son compte pour
gérer sa boutique en conversation. Repris du pattern "mcp-oauth-maison"
d'[Elevia](https://github.com/gilmry/elevia), étendu en **lecture et
écriture** : Elevia s'arrêtait au read-only, ici les outils MCP peuvent
publier un panier, le modifier, le marquer écoulé, valider un retrait et
mettre à jour le profil marchand, en plus de la lecture.

Voir les commentaires en tête de
[`backend/src/infrastructure/web/oauth.rs`](../backend/src/infrastructure/web/oauth.rs)
et [`backend/src/infrastructure/web/mcp.rs`](../backend/src/infrastructure/web/mcp.rs)
pour le détail des décisions de sécurité (PKCE S256 uniquement, refresh
token jamais stocké en clair, rotation inconditionnelle, `redirect_uri`
jamais fait confiance avant validation, `tools/call` qui revérifie
l'appartenance du panier indépendamment de `tools/list`).

## Authentification

- Un seul type de compte peut se connecter : **marchand** (même table,
  mêmes identifiants que `/marchands/connexion`).
- Le token d'accès émis est le même JWT que l'API REST classique, juste
  plus court (1h) et rafraîchissable (30 jours) - aucun handler REST n'a
  besoin d'un traitement spécial pour un token venu du flow MCP.
- Schéma de données : `oauth_clients`, `oauth_authorization_codes`,
  `oauth_refresh_tokens` (migration
  [`0006_oauth_mcp.sql`](../backend/migrations/0006_oauth_mcp.sql)).

## Outils MCP exposés

| Outil | Lecture/Écriture | Description |
|---|---|---|
| `list_my_produits` | lecture | Liste tous mes paniers |
| `get_my_dashboard` | lecture | Paniers sauvés / chiffre récupéré du jour |
| `get_my_profile` | lecture | Mon profil marchand |
| `publish_produit` | écriture | Publie un nouveau panier en démarque |
| `update_produit` | écriture | Modifie un de mes paniers |
| `marquer_ecoule` | écriture | Marque un panier écoulé |
| `valider_retrait` | écriture | Valide un code de retrait en boutique |
| `update_my_profile` | écriture | Met à jour nom/adresse/catégorie |

Les outils d'écriture s'appuient sur la confirmation d'appel d'outil du
client MCP (Claude Desktop/Code/claude.ai demandent une validation avant
d'exécuter un outil qui modifie des données) plutôt que sur une seconde
confirmation côté serveur.

## Connexion d'un client MCP

```
https://api.derniere-chance.ecosolva.org/mcp
```

Le client découvre automatiquement le flow OAuth via
`GET /.well-known/oauth-authorization-server`, s'enregistre lui-même
(`POST /oauth/register`, RFC 7591), puis lance le flow "Connect" habituel
(PKCE S256 obligatoire). Se connecter avec les identifiants marchand
habituels (email/mot de passe).

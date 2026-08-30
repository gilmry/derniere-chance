#!/usr/bin/env bash
# Sur le serveur cible (Debian/Ubuntu), une fois : ./deploy.sh
#   -> installe docker, docker compose, git, cron, crée .env si absent,
#      programme un cron qui appelle ce même script avec --run.
# Le cron appelle ensuite : ./deploy.sh --run
#   -> pull + redeploy prod si origin/main a bougé, sinon ne fait rien.
# Idempotent : les deux modes peuvent être relancés sans risque.
#
# Contrairement à Elevia, ce script ne construit RIEN localement : les images
# sont buildées en CI et poussées sur GHCR (voir .github/workflows/docker-publish.yml).
# Le serveur ne fait que `pull` + `up`. Le réseau externe `ecosolva-web` (Traefik
# partagé avec Elevia) doit déjà exister avant le premier déploiement.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRON_SCHEDULE="${CRON_SCHEDULE:-*/5 * * * *}"
CRON_MARKER="derniere-chance-auto-deploy"
LOG_FILE="$REPO_DIR/deploy.log"
LOCK_FILE="$REPO_DIR/.deploy.lock"
DEPLOYED_REV_FILE="$REPO_DIR/.deployed_rev"

log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*" >> "$LOG_FILE"; }

run_deploy() {
  cd "$REPO_DIR"

  # `docker` est un SNAP : le binaire vit dans `/snap/bin`. Le PATH de cron le
  # contient aujourd'hui, mais un déploiement automatique ne devrait pas en
  # dépendre en silence.
  case ":$PATH:" in
    *:/snap/bin:*) ;;
    *) PATH="/snap/bin:$PATH" ;;
  esac
  export PATH

  # Échouer BRUYAMMENT si docker est injoignable, au lieu de mourir muet.
  #
  # Sans ce contrôle, `running="$(docker compose …)"` plus bas échoue, `set -e`
  # tue le script AVANT sa première écriture de journal, et la panne devient
  # indiscernable d'un déploiement qui n'avait rien à faire. C'est exactement
  # ce qui s'est produit du 2026-08-27 au 2026-08-29 : socket docker recréé en
  # `root:root`, aucun groupe `docker` sur la machine, et les quatre projets du
  # VPS muets pendant deux jours, sans un seul fichier de log pour le dire.
  if ! docker info >/dev/null 2>&1; then
    log "ERREUR: docker injoignable, déploiement impossible"
    exit 1
  fi

  exec 9>"$LOCK_FILE"
  flock -n 9 || exit 0

  git fetch origin main --quiet

  local_rev="$(git rev-parse main)"
  remote_rev="$(git rev-parse origin/main)"
  running="$(docker compose --profile prod ps --status running -q 2>/dev/null)"
  deployed_rev="$(cat "$DEPLOYED_REV_FILE" 2>/dev/null || true)"

  if [ "$deployed_rev" = "$remote_rev" ] && [ -n "$running" ]; then
    exit 0
  fi

  if [ "$local_rev" = "$remote_rev" ]; then
    log "prod non déployée sur $remote_rev, déploiement"
  else
    log "nouveau commit sur main ($local_rev -> $remote_rev), déploiement"
  fi

  if ! git checkout main --quiet || ! git merge --ff-only origin/main --quiet; then
    log "échec du fast-forward vers origin/main, déploiement annulé"
    exit 1
  fi

  if ! docker compose --profile prod pull >> "$LOG_FILE" 2>&1; then
    log "échec du pull des images GHCR ($remote_rev), voir logs ci-dessus - nouvelle tentative au prochain tick"
    exit 1
  fi

  if docker compose --profile prod up -d --remove-orphans >> "$LOG_FILE" 2>&1; then
    log "déploiement réussi ($remote_rev)"
    echo "$remote_rev" > "$DEPLOYED_REV_FILE"
    docker image prune -f >> "$LOG_FILE" 2>&1
  else
    log "échec du déploiement ($remote_rev), voir logs ci-dessus - nouvelle tentative au prochain tick"
    exit 1
  fi
}

bootstrap() {
  if [ "$(id -u)" -ne 0 ] && ! command -v sudo >/dev/null 2>&1; then
    echo "root ou sudo requis pour installer les paquets système" >&2
    exit 1
  fi
  local sudo=""
  [ "$(id -u)" -ne 0 ] && sudo="sudo"

  if ! command -v apt-get >/dev/null 2>&1; then
    echo "ce script suppose une distribution basée sur apt (Debian/Ubuntu)" >&2
    exit 1
  fi

  echo "==> installation des dépendances système"
  $sudo apt-get update -qq
  $sudo apt-get install -y -qq ca-certificates curl git cron >/dev/null

  if ! command -v docker >/dev/null 2>&1; then
    echo "==> installation de Docker (script officiel get.docker.com)"
    curl -fsSL https://get.docker.com | $sudo sh
  fi

  if ! $sudo docker compose version >/dev/null 2>&1; then
    echo "docker compose (plugin v2) introuvable après installation de Docker" >&2
    exit 1
  fi

  local target_user="${SUDO_USER:-$(id -un)}"
  if [ "$target_user" != "root" ] && ! id -nG "$target_user" | grep -qw docker; then
    echo "==> ajout de $target_user au groupe docker"
    $sudo usermod -aG docker "$target_user"
    echo "note : reconnecte-toi (ou 'newgrp docker') pour que ça prenne effet dans ce shell ;"
    echo "       les futures tâches cron en tiendront compte automatiquement (nouveau process)"
  fi

  if [ ! -f "$REPO_DIR/.env" ]; then
    echo "==> création de .env depuis .env.example (à éditer avant le premier déploiement)"
    cp "$REPO_DIR/.env.example" "$REPO_DIR/.env"
  fi

  if ! docker network inspect ecosolva-web >/dev/null 2>&1; then
    echo "ATTENTION : le réseau externe 'ecosolva-web' n'existe pas encore." >&2
    echo "Démarrer d'abord le Traefik partagé (infra/shared-traefik) avant le premier déploiement." >&2
  fi

  touch "$LOG_FILE"

  echo "==> application de la politique de rétention des journaux (RGPD)"
  # Idempotent, et volontairement non bloquant : un serveur sans systemd ou
  # sans logrotate ne doit pas empêcher le déploiement. Le script rapporte ce
  # qu'il n'a pas pu faire, à traiter à la main le cas échéant.
  if ! "$REPO_DIR/infra/log-retention/install.sh"; then
    echo "ATTENTION : la politique de rétention n'a pas pu être appliquée," >&2
    echo "            relancer infra/log-retention/install.sh à la main." >&2
  fi

  echo "==> programmation du déploiement auto (cron, toutes les $CRON_SCHEDULE)"
  local cron_line="$CRON_SCHEDULE $REPO_DIR/deploy.sh --run # $CRON_MARKER"
  local existing_crontab
  existing_crontab="$(crontab -l 2>/dev/null || true)"
  local new_crontab
  if echo "$existing_crontab" | grep -qF "$CRON_MARKER"; then
    new_crontab="$(echo "$existing_crontab" | grep -vF "$CRON_MARKER")"
  else
    new_crontab="$existing_crontab"
  fi
  { echo "$new_crontab"; echo "$cron_line"; } | grep -v '^$' | crontab -

  $sudo systemctl enable --now cron >/dev/null 2>&1 || true

  cat <<EOF

Bootstrap terminé.
- Dépendances installées : docker, docker compose plugin, git, cron
- Cron installé : $cron_line
- Logs de déploiement : $LOG_FILE (tournés, 30 jours max)
- Rétention des journaux : 30 jours (voir infra/log-retention/)

Avant le premier déploiement :
1. Vérifier que le Traefik partagé tourne et que le réseau 'ecosolva-web' existe.
2. Éditer $REPO_DIR/.env (DOMAIN, API_DOMAIN, JWT_SECRET, mots de passe Postgres).
3. Soit attendre le prochain tick cron, soit lancer manuellement :
   $REPO_DIR/deploy.sh --run
EOF
}

if [ "${1:-}" = "--run" ]; then
  run_deploy
else
  bootstrap
fi

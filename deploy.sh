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

# Images GHCR à épingler. La CI les publie sous `sha-<court>` en plus de
# `latest` (voir .github/workflows/docker-publish.yml).
IMAGE_REPO="${IMAGE_REPO:-ghcr.io/gilmry/derniere-chance}"
IMAGE_SERVICES="backend frontend"

log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*" >> "$LOG_FILE"; }

# Vraie ou fausse selon que le tag existe dans le registre, sans le tirer.
image_published() {
  docker manifest inspect "$1" >/dev/null 2>&1
}

# Exporte BACKEND_IMAGE / FRONTEND_IMAGE épinglés au tag donné, pour que
# `docker compose` résolve des images précises et non un `latest` mouvant.
pin_images() {
  local tag="$1" service var
  for service in $IMAGE_SERVICES; do
    var="$(echo "$service" | tr '[:lower:]' '[:upper:]')_IMAGE"
    export "$var=$IMAGE_REPO/$service:$tag"
  done
}

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

  local target_tag="sha-$(printf %s "$remote_rev" | cut -c1-7)"

  # ATTENDRE que TOUTES les images du commit visé soient publiées.
  #
  # Sans ce contrôle, le déploiement tirait `latest`, qui n'est publié qu'à la
  # fin de chaque job de build. Quand le front et le back finissent à quelques
  # minutes d'écart, un tic de cron tombant entre les deux déployait un
  # `latest` moitié neuf moitié vieux, puis inscrivait le commit comme
  # déployé : plus aucun tic ultérieur ne rattrapait l'écart. C'est arrivé
  # deux fois le 2026-08-30, dont une en laissant tourner un frontend qui
  # appelait des routes absentes du backend.
  #
  # On sort en 0 et non en erreur : la CI n'a simplement pas fini, le
  # prochain tic réessaiera. Ce n'est pas une panne.
  local service missing=""
  for service in $IMAGE_SERVICES; do
    image_published "$IMAGE_REPO/$service:$target_tag" || missing="$missing $service"
  done
  if [ -n "$missing" ]; then
    if [ "$deployed_rev" != "$remote_rev" ]; then
      log "images $target_tag pas encore publiées ($(echo $missing)), attente du prochain tic"
    fi
    exit 0
  fi

  if [ "$local_rev" = "$remote_rev" ]; then
    log "prod non déployée sur $remote_rev, déploiement de $target_tag"
  else
    log "nouveau commit sur main ($local_rev -> $remote_rev), déploiement de $target_tag"
  fi

  if ! git checkout main --quiet || ! git merge --ff-only origin/main --quiet; then
    log "échec du fast-forward vers origin/main, déploiement annulé"
    exit 1
  fi

  pin_images "$target_tag"

  if ! docker compose --profile prod pull >> "$LOG_FILE" 2>&1; then
    log "échec du pull de $target_tag, nouvelle tentative au prochain tic"
    exit 1
  fi

  if docker compose --profile prod up -d --remove-orphans >> "$LOG_FILE" 2>&1; then
    log "déploiement réussi ($remote_rev, images $target_tag)"
    echo "$remote_rev" > "$DEPLOYED_REV_FILE"
    docker image prune -f >> "$LOG_FILE" 2>&1
    exit 0
  fi

  log "ÉCHEC du déploiement de $target_tag ($remote_rev)"
  rollback "$deployed_rev"
  exit 1
}

# Remet la version précédente en service. Sans cela, un `up -d` à moitié
# appliqué laisse la prod dans un état mixte jusqu'à intervention manuelle -
# le pire des deux mondes, puisque les conteneurs répondent mais ne
# s'accordent pas.
rollback() {
  local previous_rev="$1"

  if [ -z "$previous_rev" ]; then
    log "ROLLBACK impossible : aucune version précédente connue, intervention manuelle requise"
    return
  fi

  local previous_tag="sha-$(printf %s "$previous_rev" | cut -c1-7)"
  local service
  for service in $IMAGE_SERVICES; do
    if ! image_published "$IMAGE_REPO/$service:$previous_tag"; then
      log "ROLLBACK impossible : image $service:$previous_tag absente du registre, intervention manuelle requise"
      return
    fi
  done

  log "ROLLBACK vers $previous_tag ($previous_rev)"
  pin_images "$previous_tag"

  # Le dépôt est déjà passé sur le nouveau commit ; on ne le rembobine pas.
  # Seules les images comptent pour ce qui tourne, et `.deployed_rev` est
  # remis à la version réellement en service pour que le prochain tic
  # retente le déploiement plutôt que de croire le travail fait.
  if docker compose --profile prod up -d --remove-orphans >> "$LOG_FILE" 2>&1; then
    echo "$previous_rev" > "$DEPLOYED_REV_FILE"
    log "ROLLBACK réussi, la prod tourne sur $previous_tag"
  else
    log "ROLLBACK ÉCHOUÉ, la prod est dans un état incertain, intervention manuelle requise"
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

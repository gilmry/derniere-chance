#!/usr/bin/env bash
# Applique la politique de rétention des journaux sur le serveur cible.
#
# Appelé automatiquement par `./deploy.sh` (bootstrap), et relançable seul à
# tout moment : le script est idempotent.
#
# Ce qu'il fait :
#   1. borne journald à 30 jours (auth.log, sshd, Fail2ban, unités
#      Suricata/CrowdSec : que des adresses IP, sans limite de temps par
#      défaut) ;
#   2. installe une configuration logrotate pour CrowdSec et pour le journal
#      de déploiement, en sautant tout chemin déjà couvert ailleurs - deux
#      stanzas pour le même fichier font échouer logrotate ;
#   3. vérifie que les autres journaux de sécurité (Suricata, Fail2ban, AIDE,
#      journaux système) sont bien purgés sous 30 jours, et le signale sinon.
#
# Voir docs/rgpd/registre-traitements.md pour les durées et leur
# justification.
set -euo pipefail

MAX_RETENTION_DAYS=30
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
DEPLOY_LOG="$REPO_DIR/deploy.log"
DEPLOY_USER="$(stat -c '%U' "$REPO_DIR")"
LOGROTATE_TARGET="/etc/logrotate.d/zz-derniere-chance-retention"
JOURNALD_TARGET="/etc/systemd/journald.conf.d/derniere-chance-retention.conf"

SUDO=""
if [ "$(id -u)" -ne 0 ]; then
  command -v sudo >/dev/null 2>&1 || { echo "root ou sudo requis" >&2; exit 1; }
  SUDO="sudo"
fi

info() { echo "==> $*"; }
warn() { echo "ATTENTION : $*" >&2; }

# Un chemin déjà réclamé par un autre fichier de /etc/logrotate.d ne doit pas
# être repris ici, sous peine de "duplicate log entry" au prochain passage.
covered_elsewhere() {
  local path="$1" file
  for file in /etc/logrotate.d/*; do
    [ -f "$file" ] || continue
    [ "$file" = "$LOGROTATE_TARGET" ] && continue
    grep -qF -- "$path" "$file" && { echo "$file"; return 0; }
  done
  return 1
}

# --- 1. journald ---------------------------------------------------------

info "rétention journald : $MAX_RETENTION_DAYS jours"
$SUDO install -d -m 0755 /etc/systemd/journald.conf.d
$SUDO install -m 0644 "$SCRIPT_DIR/journald-retention.conf" "$JOURNALD_TARGET"
if $SUDO systemctl restart systemd-journald >/dev/null 2>&1; then
  # La limite de temps ne s'applique qu'à la rotation suivante : on force une
  # purge immédiate pour que l'existant soit tout de suite conforme.
  $SUDO journalctl --vacuum-time="${MAX_RETENTION_DAYS}d" >/dev/null 2>&1 || true
else
  warn "systemd-journald n'a pas redémarré, la nouvelle rétention s'appliquera au prochain démarrage"
fi

# --- 2. logrotate --------------------------------------------------------

tmp="$(mktemp)"
trap 'rm -f "$tmp" "$tmp.pruned"' EXIT
sed -e "s|@DEPLOY_LOG@|$DEPLOY_LOG|g" -e "s|@DEPLOY_USER@|$DEPLOY_USER|g" \
  "$SCRIPT_DIR/logrotate-derniere-chance" > "$tmp"

# Retire du fichier généré la stanza dont le titre est passé en argument.
drop_stanza() {
  local title="$1"
  awk -v title="# --- $title" '
    index($0, title) == 1 { skipping = 1 }
    skipping && /^\}$/   { skipping = 0; just_closed = 1; next }
    skipping              { next }
    just_closed && /^$/   { just_closed = 0; next }
                          { just_closed = 0; print }
  ' "$tmp" > "$tmp.pruned" && mv "$tmp.pruned" "$tmp"
}

# CrowdSec n'est pas toujours installé : sans lui la stanza ne sert à rien,
# et son `missingok` masquerait un jour une vraie absence de rotation.
if [ ! -e /var/log/crowdsec.log ] && [ ! -d /etc/crowdsec ]; then
  info "CrowdSec absent, bloc correspondant retiré"
  drop_stanza "CrowdSec"
elif owner="$(covered_elsewhere /var/log/crowdsec.log)"; then
  warn "/var/log/crowdsec.log est déjà géré par $owner, bloc non installé - vérifier sa rétention"
  drop_stanza "CrowdSec"
fi

$SUDO install -m 0644 "$tmp" "$LOGROTATE_TARGET"

if ! $SUDO logrotate --debug "$LOGROTATE_TARGET" >/dev/null 2>&1; then
  $SUDO rm -f "$LOGROTATE_TARGET"
  echo "configuration logrotate invalide, installation annulée :" >&2
  $SUDO logrotate --debug "$tmp" >&2 || true
  exit 1
fi
info "logrotate installé dans $LOGROTATE_TARGET"

# --- 3. audit des journaux gérés ailleurs --------------------------------

# Traduit la fréquence et le nombre de rotations d'une stanza en jours de
# conservation, pour comparer au plafond.
audit_path() {
  local path="$1" label="$2" owner
  if ! owner="$(covered_elsewhere "$path")"; then
    warn "$label ($path) n'est couvert par aucune rotation - conservation potentiellement indéfinie"
    return
  fi

  local rotate frequency days
  rotate="$(grep -oP '^\s*rotate\s+\K[0-9]+' "$owner" | head -1 || true)"
  frequency="$(grep -oP '^\s*\K(daily|weekly|monthly)' "$owner" | head -1 || true)"
  [ -n "$rotate" ] || { info "$label : géré par $owner (rétention non déterminée automatiquement)"; return; }

  case "${frequency:-weekly}" in
    daily) days=$((rotate)) ;;
    weekly) days=$((rotate * 7)) ;;
    monthly) days=$((rotate * 31)) ;;
  esac

  if [ "$days" -gt "$MAX_RETENTION_DAYS" ]; then
    warn "$label : ~$days jours via $owner, au-delà du plafond de $MAX_RETENTION_DAYS jours - à resserrer"
  else
    info "$label : ~$days jours via $owner, conforme"
  fi
}

audit_path /var/log/suricata "Suricata"
audit_path /var/log/fail2ban.log "Fail2ban"
audit_path /var/log/auth.log "Journaux système (rsyslog)"

# AIDE ne passe pas par logrotate : son lanceur quotidien fait tourner le
# journal lui-même (`savelog -c 7`), soit 7 jours.
if [ -d /var/log/aide ]; then
  if grep -q 'savelog .* -c 7' /usr/share/aide/bin/dailyaidecheck 2>/dev/null; then
    info "AIDE : ~7 jours via savelog (dailyaidecheck), conforme"
  else
    warn "AIDE : rotation non reconnue, vérifier /usr/share/aide/bin/dailyaidecheck"
  fi
fi

info "politique de rétention appliquée (plafond : $MAX_RETENTION_DAYS jours)"

# Rétention des journaux

Politique de conservation des journaux du serveur qui héberge
DernièreChance. Elle existe pour une raison précise : les journaux
contiennent des adresses IP, donc des données personnelles, et le RGPD
interdit de les garder indéfiniment. Le détail des durées et leur
justification sont dans
[`docs/rgpd/registre-traitements.md`](../../docs/rgpd/registre-traitements.md).

**Plafond : 30 jours**, tous journaux confondus.

## Installation

`./deploy.sh` l'applique automatiquement au bootstrap. Pour la (ré)appliquer
seule, le script étant idempotent :

```sh
./infra/log-retention/install.sh
```

Il faut root ou sudo (écriture dans `/etc/systemd/journald.conf.d` et
`/etc/logrotate.d`).

## Ce qui est appliqué

| Journal | Durée | Par quoi |
|---|---|---|
| journald (auth, sshd, Fail2ban, unités Suricata/CrowdSec) | 30 jours, plafonné à 500 Mo | `journald-retention.conf` installé par ce script |
| CrowdSec | 30 jours | `logrotate-derniere-chance`, installé seulement si CrowdSec est présent |
| `deploy.log` | 4 semaines, `maxage 30` | `logrotate-derniere-chance` |
| Conteneurs Docker (backend, frontend, postgres, minio) | 50 Mo par service | `logging: *journaux` dans `docker-compose.yml` |
| Suricata | 7 jours | Configuration de la distribution, **vérifiée** par le script |
| Fail2ban | 4 semaines | Configuration de la distribution, **vérifiée** |
| Journaux système (rsyslog) | 4 semaines | Configuration de la distribution, **vérifiée** |
| AIDE | 7 jours | `savelog -c 7` dans `dailyaidecheck`, **vérifié** |

Les lignes « vérifiées » ne sont pas réécrites : deux configurations
logrotate visant le même fichier font échouer la rotation entière
(*duplicate log entry*), ce qui reviendrait à ne plus rien purger du tout. Le
script se contente de mesurer la rétention effective de ces journaux et
d'avertir si elle dépasse 30 jours, auquel cas il faut resserrer le fichier
de la distribution à la main.

## Vérifier l'état sur le serveur

```sh
journalctl --disk-usage
sudo journalctl --header | grep -i retention
sudo logrotate --debug /etc/logrotate.d/zz-derniere-chance-retention
```

// Consentement au programme bêta : constantes partagées et redirection.
//
// Ce module ne dépend de rien d'autre à dessein : api.ts l'importe pour
// rebasculer vers l'écran de consentement, donc y importer api.ts en retour
// créerait un cycle.

/// Version du texte de consentement en vigueur. À tenir synchronisée avec
/// `BETA_CONSENT_VERSION` côté backend
/// (backend/src/application/use_cases/consent_use_cases.rs) et avec la date
/// de mise à jour affichée sur /confidentialite : le backend refuse une
/// inscription qui accepte une version différente de la sienne.
export const BETA_CONSENT_VERSION = "2026-08-30";

export const CONSENT_PATH = "/consentement";

// Pages atteignables sans consentement : l'accueil et l'inscription (on n'a
// pas encore pu consentir), les pages légales (les lire est justement le
// préalable), et les espaces marchand/admin, qui relèvent d'autres
// principaux et ne sont pas concernés par le programme bêta consommateur.
const EXEMPT_PATHS = ["/", "/compte", CONSENT_PATH, "/confidentialite", "/mentions-legales"];
const EXEMPT_PREFIXES = ["/pro/", "/admin"];

/// Astro sert les pages statiques indifféremment en `/compte` ou `/compte/`,
/// d'où la normalisation avant comparaison.
export function isConsentExempt(pathname: string): boolean {
  const path = pathname.replace(/\/+$/, "") || "/";
  return EXEMPT_PATHS.includes(path) || EXEMPT_PREFIXES.some((prefix) => path.startsWith(prefix));
}

export function redirectToConsent(): void {
  if (typeof window === "undefined") return;
  if (isConsentExempt(window.location.pathname)) return;
  const next = window.location.pathname + window.location.search;
  window.location.href = `${CONSENT_PATH}?next=${encodeURIComponent(next)}`;
}

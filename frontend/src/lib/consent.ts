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

/// Un écran par principal : les jetons marchand et consommateur ne sont pas
/// interchangeables, donc le retour vers le bon espace non plus.
export const CONSENT_PATH = "/consentement";
export const MERCHANT_CONSENT_PATH = "/pro/consentement";

// Pages atteignables sans consentement : l'accueil et les écrans
// d'inscription/connexion (on n'a pas encore pu consentir), les pages
// légales (les lire est justement le préalable), et les deux écrans de
// consentement eux-mêmes. L'espace admin relève d'un principal qui n'est pas
// concerné par le programme bêta.
const EXEMPT_PATHS = [
  "/",
  "/compte",
  "/confidentialite",
  "/mentions-legales",
  "/pro/login",
  CONSENT_PATH,
  MERCHANT_CONSENT_PATH,
];
const EXEMPT_PREFIXES = ["/admin"];

/// Astro sert les pages statiques indifféremment en `/compte` ou `/compte/`,
/// d'où la normalisation avant comparaison.
export function isConsentExempt(pathname: string): boolean {
  const path = pathname.replace(/\/+$/, "") || "/";
  return EXEMPT_PATHS.includes(path) || EXEMPT_PREFIXES.some((prefix) => path.startsWith(prefix));
}

/// Vrai pour les pages de l'espace marchand, qui relèvent du jeton marchand.
export function isMerchantPath(pathname: string): boolean {
  return pathname.replace(/\/+$/, "").startsWith("/pro");
}

export function redirectToConsent(path = CONSENT_PATH): void {
  if (typeof window === "undefined") return;
  if (isConsentExempt(window.location.pathname)) return;
  const next = window.location.pathname + window.location.search;
  window.location.href = `${path}?next=${encodeURIComponent(next)}`;
}

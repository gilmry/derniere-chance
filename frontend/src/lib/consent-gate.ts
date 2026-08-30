// Portier de consentement côté navigateur, appelé au chargement de chaque
// page par AppLayout.
//
// Le vrai verrou est côté backend (extracteurs `ConsentedConsumer` et
// `ConsentedMerchant` : tout appel authentifié est refusé sans consentement à
// jour). Celui-ci évite seulement d'afficher à un testeur non consentant une
// application en apparence utilisable qui échouerait à chaque action.

import { consentStatus } from "./api";
import { getConsumerToken, getMerchantToken } from "./auth";
import {
  CONSENT_PATH,
  MERCHANT_CONSENT_PATH,
  isConsentExempt,
  isMerchantPath,
  redirectToConsent,
} from "./consent";

export async function enforceConsentGate(): Promise<void> {
  if (typeof window === "undefined") return;
  if (isConsentExempt(window.location.pathname)) return;

  // Le chemin décide du principal : une même personne peut détenir les deux
  // jetons dans le même navigateur (c'est le cas des tests e2e), donc lire
  // « le jeton présent » ne suffirait pas à choisir.
  const merchantSide = isMerchantPath(window.location.pathname);
  const token = merchantSide ? getMerchantToken() : getConsumerToken();
  if (!token) return;

  try {
    const status = await consentStatus(token, merchantSide ? "marchand" : "consommateur");
    if (!status.consenti) {
      redirectToConsent(merchantSide ? MERCHANT_CONSENT_PATH : CONSENT_PATH);
    }
  } catch {
    // Jeton expiré ou backend injoignable : on laisse la page se charger.
    // Les composants gèrent déjà leurs propres erreurs, et le backend reste
    // le garde-fou qui refusera les actions.
  }
}

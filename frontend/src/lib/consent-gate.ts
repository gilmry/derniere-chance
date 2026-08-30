// Portier de consentement côté navigateur, appelé au chargement de chaque
// page par AppLayout.
//
// Le vrai verrou est côté backend (extracteur `ConsentedConsumer` : tout
// appel consommateur est refusé sans consentement à jour). Celui-ci évite
// seulement d'afficher à un testeur non consentant une application en
// apparence utilisable qui échouerait à chaque action.

import { consentStatus } from "./api";
import { getConsumerToken } from "./auth";
import { isConsentExempt, redirectToConsent } from "./consent";

export async function enforceConsentGate(): Promise<void> {
  if (typeof window === "undefined") return;
  if (isConsentExempt(window.location.pathname)) return;

  const token = getConsumerToken();
  if (!token) return;

  try {
    const status = await consentStatus(token);
    if (!status.consenti) redirectToConsent();
  } catch {
    // Jeton expiré ou backend injoignable : on laisse la page se charger.
    // Les composants gèrent déjà leurs propres erreurs, et le backend reste
    // le garde-fou qui refusera les actions.
  }
}

// Static build (output:"static"): dynamic segments aren't known at build
// time, so id/code/mode live in the query string and get read client-side.
export function getQueryParam(name: string): string | null {
  if (typeof window === "undefined") return null;
  return new URLSearchParams(window.location.search).get(name);
}

/// Adresse de retour (`?next=`) acceptée seulement si elle reste sur le site :
/// un lien piégé ne doit pas pouvoir renvoyer vers un autre site
/// (« //exemple.com », « https://… ») ni exécuter du code (« javascript: »).
export function pageInterne(next: string | null): string | null {
  if (!next || !next.startsWith("/") || next.startsWith("//") || next.startsWith("/\\")) {
    return null;
  }
  return next;
}

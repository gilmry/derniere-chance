// Static build (output:"static"): dynamic segments aren't known at build
// time, so id/code/mode live in the query string and get read client-side.
export function getQueryParam(name: string): string | null {
  if (typeof window === "undefined") return null;
  return new URLSearchParams(window.location.search).get(name);
}

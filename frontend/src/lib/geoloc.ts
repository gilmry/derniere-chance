// Position du navigateur, en best-effort : refus, timeout ou absence d'API
// renvoient simplement null plutôt que de bloquer l'UI (cf. VISION.md §8).
export interface Coords {
  lat: number;
  lon: number;
}

export function getBrowserPosition(timeoutMs = 5000): Promise<Coords | null> {
  return new Promise((resolve) => {
    if (typeof navigator === "undefined" || !navigator.geolocation) {
      resolve(null);
      return;
    }
    navigator.geolocation.getCurrentPosition(
      (pos) => resolve({ lat: pos.coords.latitude, lon: pos.coords.longitude }),
      () => resolve(null),
      { timeout: timeoutMs, maximumAge: 60_000 },
    );
  });
}

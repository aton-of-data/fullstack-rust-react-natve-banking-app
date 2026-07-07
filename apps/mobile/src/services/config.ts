/**
 * Resolves the API base URL from environment with localhost fallback.
 *
 * @returns Base URL without trailing slash.
 */
export function getApiBaseUrl(): string {
  const raw = process.env.EXPO_PUBLIC_API_URL ?? 'http://localhost:8080';
  return raw.replace(/\/$/, '');
}

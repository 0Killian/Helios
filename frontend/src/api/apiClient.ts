export class ApiError extends Error {
  public code: string;
  public message: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = "ApiError";
    this.code = code;
    this.message = message;
  }
}

const baseUrl = import.meta.env.VITE_API_URL;

/**
 * A generic API client to interact with the Helios backend.
 * @param endpoint The API endpoint to call (e.g., '/api/v1/services').
 * @param options Standard fetch options (method, headers, body).
 * @returns The `data` portion of the successful API response.
 * @throws {ApiError} If the API returns a `success: false` response.
 */
export async function apiClient<T>(
  endpoint: string,
  options?: RequestInit,
): Promise<T> {
  const response = await fetch(baseUrl + endpoint, options);

  const contentType = response.headers.get("content-type");
  if (!contentType || !contentType.includes("application/json")) {
    if (!response.ok) {
      const textError = await response.text();
      throw new Error(textError || "An unknown server error occurred.");
    }
    throw new Error("Invalid response format");
  }

  const payload = await response.json();

  if (payload.success) {
    return payload.data as T;
  } else {
    throw new ApiError(payload.error.code, payload.error.message);
  }
}

import { z } from "zod";

const API_BASE =
  (import.meta.env.VITE_API_BASE_URL as string) || "http://127.0.0.1:8081";

export const HealthSchema = z.object({
  status: z.string(),
  database: z.string(),
});

export async function getHealth(): Promise<z.infer<typeof HealthSchema>> {
  const res = await fetch(`${API_BASE}/health`);
  if (!res.ok) throw new Error(`health check failed: ${res.status}`);
  const json = await res.json();
  return HealthSchema.parse(json);
}

function buildAuthHeader(token: string) {
  if (!token || token.trim().length === 0) throw new Error("token empty");
  if (token.includes(" ")) throw new Error("token must not contain spaces");
  return `Bearer ${token}`;
}

export async function getQuota(
  token: string,
): Promise<{ user_id: string; quota_bytes: number; used_bytes: number }> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (token) headers["Authorization"] = buildAuthHeader(token);

  const res = await fetch(`${API_BASE}/api/quota`, { headers });
  if (res.status === 401) throw new Error("unauthorized");
  if (!res.ok) throw new Error(`quota fetch failed: ${res.status}`);
  const json = await res.json();
  return {
    user_id: String(json.user_id),
    quota_bytes: Number(json.quota_bytes),
    used_bytes: Number(json.used_bytes),
  };
}

export default { getHealth, getQuota };

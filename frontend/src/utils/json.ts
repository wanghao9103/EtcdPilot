export type JsonResult =
  | { ok: true; value: string }
  | { ok: false; error: string };

export function looksLikeJson(raw: string): boolean {
  const text = raw.trim();
  if (!text) return false;
  const first = text[0];
  const last = text[text.length - 1];
  return (first === "{" && last === "}") || (first === "[" && last === "]");
}

export function validateJson(raw: string): { valid: boolean; error?: string } {
  const text = raw.trim();
  if (!text) return { valid: true };
  try {
    JSON.parse(text);
    return { valid: true };
  } catch (err) {
    return { valid: false, error: (err as Error).message };
  }
}

export function formatJson(raw: string, indent = 2): JsonResult {
  try {
    const parsed = JSON.parse(raw);
    return { ok: true, value: JSON.stringify(parsed, null, indent) };
  } catch (err) {
    return { ok: false, error: (err as Error).message };
  }
}

export function minifyJson(raw: string): JsonResult {
  try {
    const parsed = JSON.parse(raw);
    return { ok: true, value: JSON.stringify(parsed) };
  } catch (err) {
    return { ok: false, error: (err as Error).message };
  }
}

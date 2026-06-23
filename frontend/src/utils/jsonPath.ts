export function parseJsonSafe(raw: string): unknown | null {
  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

export function stringifyJson(value: unknown, pretty = true): string {
  return pretty ? JSON.stringify(value, null, 2) : JSON.stringify(value);
}

export function getValueAtPath(root: unknown, path: string[]): unknown {
  let current = root;
  for (const segment of path) {
    if (current === null || current === undefined) return undefined;
    if (Array.isArray(current)) {
      current = current[Number(segment)];
    } else if (typeof current === "object") {
      current = (current as Record<string, unknown>)[segment];
    } else {
      return undefined;
    }
  }
  return current;
}

export function setValueAtPath(root: unknown, path: string[], value: unknown): unknown {
  if (path.length === 0) return value;
  const [head, ...rest] = path;
  if (Array.isArray(root)) {
    const copy = [...root];
    const idx = Number(head);
    copy[idx] = setValueAtPath(copy[idx], rest, value);
    return copy;
  }
  if (root && typeof root === "object") {
    return {
      ...(root as Record<string, unknown>),
      [head]: setValueAtPath((root as Record<string, unknown>)[head], rest, value),
    };
  }
  return root;
}

export type JsonPrimitiveType = "string" | "number" | "boolean" | "null";

export function valueType(value: unknown): string {
  if (value === null) return "null";
  if (Array.isArray(value)) return "array";
  return typeof value;
}

export function isStructuredJson(value: unknown): value is Record<string, unknown> | unknown[] {
  return value !== null && typeof value === "object";
}

export function previewValue(value: unknown, max = 48): string {
  if (value === null) return "null";
  if (typeof value === "string") {
    const text = value.length > max ? `${value.slice(0, max)}…` : value;
    return `"${text}"`;
  }
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  if (Array.isArray(value)) return `[${value.length}]`;
  if (typeof value === "object") return `{${Object.keys(value as object).length}}`;
  return String(value);
}

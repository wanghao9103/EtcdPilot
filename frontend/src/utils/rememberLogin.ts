const STORAGE_KEY = "etcdpilot.login.remember";

export interface RememberedLogin {
  remember: boolean;
  username: string;
  password: string;
}

const encode = (value: string) => btoa(unescape(encodeURIComponent(value)));
const decode = (value: string) => decodeURIComponent(escape(atob(value)));

export function loadRememberedLogin(): RememberedLogin | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as { remember?: boolean; username?: string; password?: string };
    if (!parsed.remember) return { remember: false, username: "", password: "" };
    return {
      remember: true,
      username: parsed.username ? decode(parsed.username) : "",
      password: parsed.password ? decode(parsed.password) : "",
    };
  } catch {
    return null;
  }
}

export function saveRememberedLogin(payload: RememberedLogin) {
  if (!payload.remember) {
    localStorage.removeItem(STORAGE_KEY);
    return;
  }
  localStorage.setItem(
    STORAGE_KEY,
    JSON.stringify({
      remember: true,
      username: encode(payload.username),
      password: encode(payload.password),
    }),
  );
}

export function clearRememberedLogin() {
  localStorage.removeItem(STORAGE_KEY);
}

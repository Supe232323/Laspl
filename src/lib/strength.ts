/** Simple password strength heuristic (no network, no deps). */

export type StrengthLevel = "empty" | "weak" | "fair" | "good" | "strong";

export interface StrengthResult {
  level: StrengthLevel;
  score: number; // 0–4
  label: string;
  color: string;
}

export function scorePassword(password: string): StrengthResult {
  if (!password) {
    return { level: "empty", score: 0, label: "", color: "bg-sn-border" };
  }

  let score = 0;
  const len = password.length;

  if (len >= 8) score++;
  if (len >= 12) score++;
  if (len >= 16) score++;
  if (len >= 20) score++;

  const hasLower = /[a-z]/.test(password);
  const hasUpper = /[A-Z]/.test(password);
  const hasDigit = /\d/.test(password);
  const hasSymbol = /[^a-zA-Z0-9]/.test(password);
  const variety = [hasLower, hasUpper, hasDigit, hasSymbol].filter(Boolean).length;
  if (variety >= 3) score++;
  if (variety >= 4) score++;

  if (password.includes("-") && password.split("-").length >= 4) score += 2;
  if (password.includes(" ") && password.split(/\s+/).length >= 4) score += 2;

  if (/^[a-z]+$/i.test(password) && len < 12) score = Math.max(0, score - 1);
  if (/^\d+$/.test(password)) score = Math.min(score, 1);
  if (/(.)\1{3,}/.test(password)) score = Math.max(0, score - 1);

  score = Math.min(4, score);

  if (score <= 1) {
    return { level: "weak", score: 1, label: "Weak", color: "bg-red-500" };
  }
  if (score === 2) {
    return { level: "fair", score: 2, label: "Fair", color: "bg-yellow-500" };
  }
  if (score === 3) {
    return { level: "good", score: 3, label: "Good", color: "bg-lime-500" };
  }
  return { level: "strong", score: 4, label: "Strong", color: "bg-emerald-500" };
}

export function findDuplicatePasswords(
  password: string,
  entries: { id: string; title: string; password: string; deleted?: boolean }[],
  excludeId?: string
): string[] {
  if (!password) return [];
  return entries
    .filter(
      (e) =>
        !e.deleted &&
        e.password === password &&
        e.id !== excludeId
    )
    .map((e) => e.title);
}

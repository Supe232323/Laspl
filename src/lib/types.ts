export interface PasswordEntry {
  id: string;
  title: string;
  username: string;
  password: string;
  url?: string;
  notes?: string;
  favorite: boolean;
  category?: string;
  /** Soft-deleted flag */
  deleted: boolean;
  /** Unix timestamp in milliseconds */
  createdAt: number;
  /** Unix timestamp in milliseconds */
  updatedAt: number;
  /** Unix timestamp in milliseconds — set when password is copied/viewed */
  lastUsedAt?: number;
}

export interface VaultMeta {
  version: number;
  createdAt: number;
  lastUnlocked?: number;
}

export type View =
  | "all"
  | "favorites"
  | "recent"
  | "trash"
  | "category";

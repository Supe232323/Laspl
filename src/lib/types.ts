export interface PasswordEntry {
  id: string;
  title: string;          // site / service name
  username: string;
  password: string;
  url?: string;
  notes?: string;
  favorite: boolean;
  category?: string;
  createdAt: number;
  updatedAt: number;
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

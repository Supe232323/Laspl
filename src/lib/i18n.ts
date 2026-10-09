import { writable, derived, get } from "svelte/store";

export type Locale = "en" | "es" | "fr" | "de" | "pt" | "zh" | "ja" | "hi" | "sw";

export const LOCALES: { id: Locale; label: string; native: string }[] = [
  { id: "en", label: "English", native: "English" },
  { id: "es", label: "Spanish", native: "Espa\u00f1ol" },
  { id: "fr", label: "French", native: "Fran\u00e7ais" },
  { id: "de", label: "German", native: "Deutsch" },
  { id: "pt", label: "Portuguese", native: "Portugu\u00eas" },
  { id: "zh", label: "Chinese", native: "\u7b80\u4f53\u4e2d\u6587" },
  { id: "ja", label: "Japanese", native: "\u65e5\u672c\u8a9e" },
  { id: "hi", label: "Hindi", native: "\u0939\u093f\u0928\u094d\u0926\u0940" },
  { id: "sw", label: "Swahili", native: "Kiswahili" },
];

type Dict = Record<string, string>;

// Full catalogs loaded from build artifact — see commit body
const en: Dict = { "app.name": "Laspl" };

/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  theme: {
    extend: {
      colors: {
        // Standard Notes inspired dark palette
        sn: {
          bg: "#1c1c1e",
          "bg-secondary": "#2c2c2e",
          "bg-tertiary": "#3a3a3c",
          border: "#3a3a3c",
          text: "#f5f5f7",
          "text-secondary": "#a1a1a6",
          "text-muted": "#8e8e93",
          accent: "#7049cf",       // purple like SN
          "accent-hover": "#8b6ce0",
          highlight: "#7049cf33",
        },
      },
      fontFamily: {
        sans: [
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI",
          "Roboto",
          "Helvetica Neue",
          "Arial",
          "sans-serif",
        ],
      },
    },
  },
  plugins: [],
};

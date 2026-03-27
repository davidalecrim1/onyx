/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        mono: ['"Zed Mono Extended"', "monospace"],
      },
      colors: {
        background: "var(--onyx-background)",
        surface: "var(--onyx-surface)",
        "surface-hover": "var(--onyx-surface-hover)",
        "surface-active": "var(--onyx-surface-active)",
        accent: "var(--onyx-accent)",
        "text-primary": "var(--onyx-text-primary)",
        "text-secondary": "var(--onyx-text-secondary)",
      },
    },
  },
  plugins: [],
};

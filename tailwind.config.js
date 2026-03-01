/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        background: "#282c33",
        surface: "#2f343e",
        "surface-hover": "#363c46",
        "surface-active": "#454a56",
        accent: "#74ade8",
        "text-primary": "#dce0e5",
        "text-secondary": "#a9afbc",
      },
    },
  },
  plugins: [],
};

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{rs,html,css}",
    "./dist/**/*.html",
  ],
  theme: {
    extend: {
      colors: {
        'light-background': '#FFFFFF', // White
        'light-foreground': '#1F2937', // Cool Gray 800 (darker text)
        'light-border': '#D1D5DB',     // Cool Gray 300 (light border)
        'light-card': '#F9FAFB',       // Cool Gray 50 (very light gray for cards/elements)
        'light-primary': '#3B82F6',    // Blue 500 (for primary actions)
        'light-secondary-text': '#6B7280', // Cool Gray 500 (for secondary text)
      },
    },
  },
  plugins: [],
};

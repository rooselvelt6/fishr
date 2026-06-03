/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./crates/fishr-agent/src/frontend/**/*.rs",
    "./crates/fishr-central/src/frontend/**/*.rs",
  ],
  theme: {
    extend: {
      colors: {
        fish: {
          blue: '#1e40af',
          teal: '#0d9488',
        }
      }
    },
  },
  plugins: [],
}

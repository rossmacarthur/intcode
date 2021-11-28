const defaultTheme = require("tailwindcss/defaultTheme");

const colors = {
  transparent: "transparent",
  current: "currentColor",
  // Tomorrow Night Eighties
  gray: {
    darker: "#2d2d2d",
    dark: "#393939",
    DEFAULT: "#515151",
    light: "#999999",
    lighter: "#cccccc",
    lightest: "#efefef",
  },
  red: "#f2777a",
  orange: "#f99157",
  yellow: "#ffcc66",
  green: "#99cc99",
  aqua: "#66cccc",
  blue: "#6699cc",
  purple: "#cc99cc",
};

module.exports = {
  mode: "jit",
  purge: ["./pages/**/*.{js,ts,jsx,tsx}", "./components/**/*.{js,ts,jsx,tsx}"],
  darkMode: false, // or 'media' or 'class'

  theme: {
    extend: {
      colors: colors,
      fontFamily: {
        mono: ["Source Code Pro", ...defaultTheme.fontFamily.mono],
      },
      animation: {
        "spin-fast": "spin 0.5s linear infinite",
      },
      outline: {
        blue: "2px solid #6699cc",
      },
    },
  },
};

/** @type {import('tailwindcss').Config} */

const defaultTheme = require("tailwindcss/defaultTheme");
const colors = require("tailwindcss/colors");

module.exports = {
  darkMode: "class",
  content: ["./templates/**/*.html"],
  theme: {
    fontFamily: {
      sans: ["Inter", ...defaultTheme.fontFamily.sans],
      mono: ["JetBrains Mono", ...defaultTheme.fontFamily.mono],
    },
    colors: {
      "bg-h": {
        light: "#f9f5d7",
        dark: "#1d2021",
      },
      "bg-primary": {
        light: "#fbf1c7",
        dark: "#282828",
      },
      "bg-s": {
        light: "#f2e5bc",
        dark: "#32302f",
      },
      "bg-1": {
        light: "#ebdbb2",
        dark: "#3c3836",
      },
      "bg-2": {
        light: "#d5c4a1",
        dark: "#504945",
      },
      "bg-3": {
        light: "#bdae93",
        dark: "#665c54",
      },
      "bg-4": {
        light: "#a89984",
        dark: "#7c6f64",
      },
      fg: {
        light: "#282828",
        dark: "#fbf1c7",
      },
      "fg-1": {
        light: "#3c3836",
        dark: "#ebdbb2",
      },
      "fg-2": {
        light: "#504945",
        dark: "#d5c4a1",
      },
      "fg-3": {
        light: "#665c54",
        dark: "#bdae93",
      },
      "fg-4": {
        light: "#7c6f64",
        dark: "#a89984",
      },
      red: {
        light: "#9d0006",
        dark: "#fb4934",
      },
      green: {
        light: "#79740e",
        dark: "#b8bb26",
      },
      yellow: {
        light: "#b57614",
        dark: "#fabd2f",
      },
      blue: {
        light: "#076678",
        dark: "#83a598",
      },
      purple: {
        light: "#8f3f71",
        dark: "#d3869b",
      },
      aqua: {
        light: "#427b58",
        dark: "#8ec07c",
      },
      orange: {
        light: "#af3a03",
        dark: "#fe8019",
      },
      gray: {
        light: "#928374",
        dark: "#928374",
      },
      "red-dim": {
        light: "#cc2412",
        dark: "#cc2412",
      },
      "green-dim": {
        light: "#98971a",
        dark: "#98971a",
      },
      "yellow-dim": {
        light: "#d79921",
        dark: "#d79921",
      },
      "blue-dim": {
        light: "#458598",
        dark: "#458588",
      },
      "purple-dim": {
        light: "#b16286",
        dark: "#b16286",
      },
      "aqua-dim": {
        light: "#689d6a",
        dark: "#689d6a",
      },
      "orange-dim": {
        light: "#d65d0e",
        dark: "#d65d0e",
      },
      "gray-dim": {
        light: "#7c6f64",
        dark: "#7c6f64",
      },
    },
  },
  plugins: [],
};

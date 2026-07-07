import { createFicusConfig } from "@ficus/eslint-config";

/** @type {import('eslint').Linter.Config[]} */
export default createFicusConfig([
  {
    ignores: [
      "node_modules/**",
      ".expo/**",
      "dist/**",
      "metro.config.js",
      "babel.config.js",
    ],
  },
  {
    files: ["**/index.ts", "**/index.tsx"],
    rules: {
      "jsdoc/require-jsdoc": "off",
    },
  },
]);

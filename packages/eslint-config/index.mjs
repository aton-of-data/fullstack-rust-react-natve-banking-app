import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import reactNative from "eslint-plugin-react-native";
import jsdoc from "eslint-plugin-jsdoc";
import prettier from "eslint-config-prettier";

/**
 * Approved bootstrap and infrastructure files where low-level React or HTTP
 * primitives may be used (e.g. Redux Provider shell, RTK Query baseApi).
 */
export const bootstrapFilePatterns = [
  "**/src/app/**/*.{ts,tsx}",
  "**/baseApi.{ts,tsx}",
  "**/*.config.{ts,tsx,mjs,js,cjs}",
  "**/eslint.config.mjs",
  "**/__tests__/**",
  "**/*.test.{ts,tsx}",
];

/** Architectural restriction rules enforced across application code. */
export const architecturalRules = {
  "no-restricted-syntax": [
    "error",
    {
      selector: "CallExpression[callee.name='useState']",
      message:
        "useState is prohibited for application state. Use Redux Toolkit.",
    },
    {
      selector: "CallExpression[callee.name='useReducer']",
      message: "useReducer is prohibited. Use Redux Toolkit.",
    },
    {
      selector: "CallExpression[callee.name='createContext']",
      message:
        "createContext is prohibited. Use Redux Toolkit (Provider exception in bootstrap files).",
    },
    {
      selector: "CallExpression[callee.name='useContext']",
      message: "useContext is prohibited. Use Redux Toolkit selectors.",
    },
    {
      selector: "CallExpression[callee.name='fetch']",
      message: "fetch is prohibited in UI layers. Use RTK Query.",
    },
  ],
  "no-restricted-imports": [
    "error",
    {
      paths: [
        { name: "axios", message: "axios is prohibited. Use RTK Query." },
      ],
    },
  ],
};

/**
 * Default Ficus ESLint flat config with architectural rules and JSDoc enforcement.
 *
 * @param {import('eslint').Linter.Config[]} [overrides] Additional config slices appended last.
 * @returns {import('eslint').Linter.Config[]}
 */
export function createFicusConfig(overrides = []) {
  return tseslint.config(
    js.configs.recommended,
    ...tseslint.configs.recommended,
    prettier,
    {
      plugins: {
        "react-hooks": reactHooks,
        "react-native": reactNative,
        jsdoc,
      },
      rules: {
        ...architecturalRules,
        "jsdoc/require-jsdoc": [
          "error",
          {
            publicOnly: true,
            require: {
              FunctionDeclaration: true,
              MethodDefinition: true,
              ClassDeclaration: true,
              ArrowFunctionExpression: false,
              FunctionExpression: false,
            },
            contexts: [
              "TSInterfaceDeclaration",
              "TSTypeAliasDeclaration",
              "ExportNamedDeclaration",
            ],
          },
        ],
        "jsdoc/require-description": "error",
        "jsdoc/require-param": "error",
        "jsdoc/require-returns": "error",
        "jsdoc/check-types": "error",
        "jsdoc/valid-types": "error",
        "jsdoc/require-param-description": "warn",
        "jsdoc/require-returns-description": "warn",
        "react-hooks/rules-of-hooks": "error",
        "react-hooks/exhaustive-deps": "warn",
      },
    },
    {
      files: bootstrapFilePatterns,
      rules: {
        "no-restricted-syntax": "off",
        "no-restricted-imports": "off",
        "jsdoc/require-jsdoc": "off",
      },
    },
    ...overrides,
  );
}

/** @type {import('eslint').Linter.Config[]} */
export default createFicusConfig();

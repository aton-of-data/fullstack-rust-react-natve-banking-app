import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import reactHooks from 'eslint-plugin-react-hooks';
import reactNative from 'eslint-plugin-react-native';
import jsdoc from 'eslint-plugin-jsdoc';
import prettier from 'eslint-config-prettier';

/** @type {import('eslint').Linter.Config[]} */
export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  prettier,
  {
    plugins: {
      'react-hooks': reactHooks,
      'react-native': reactNative,
      jsdoc,
    },
    rules: {
      // Architectural restrictions
      'no-restricted-syntax': [
        'error',
        {
          selector: "CallExpression[callee.name='useState']",
          message: 'useState is prohibited for application state. Use Redux Toolkit.',
        },
        {
          selector: "CallExpression[callee.name='useReducer']",
          message: 'useReducer is prohibited. Use Redux Toolkit.',
        },
        {
          selector: "CallExpression[callee.name='fetch']",
          message: 'fetch is prohibited in UI layers. Use RTK Query.',
        },
      ],
      'no-restricted-imports': [
        'error',
        {
          paths: [{ name: 'axios', message: 'axios is prohibited. Use RTK Query.' }],
        },
      ],

      // JSDoc enforcement on exports
      'jsdoc/require-jsdoc': [
        'error',
        {
          publicOnly: true,
          require: {
            FunctionDeclaration: true,
            MethodDefinition: true,
            ClassDeclaration: true,
            ArrowFunctionExpression: false,
            FunctionExpression: false,
          },
          contexts: ['TSInterfaceDeclaration', 'TSTypeAliasDeclaration', 'ExportNamedDeclaration'],
        },
      ],
      'jsdoc/require-description': 'error',
      'jsdoc/require-param': 'error',
      'jsdoc/require-returns': 'error',
      'jsdoc/check-types': 'error',
      'jsdoc/valid-types': 'error',
      'jsdoc/require-param-description': 'warn',
      'jsdoc/require-returns-description': 'warn',

      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'warn',
    },
  },
  {
    files: ['**/*.test.ts', '**/*.test.tsx', '**/__tests__/**'],
    rules: {
      'jsdoc/require-jsdoc': 'off',
    },
  },
  {
    files: ['*.config.*', 'eslint.config.mjs'],
    rules: {
      'jsdoc/require-jsdoc': 'off',
    },
  },
);

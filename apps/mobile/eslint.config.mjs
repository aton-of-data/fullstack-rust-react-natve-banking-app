import { createFicusConfig } from '@ficus/eslint-config';

/** @type {import('eslint').Linter.Config[]} */
export default createFicusConfig(
  [
    {
      ignores: [
        'node_modules/**',
        '.expo/**',
        'dist/**',
        'coverage/**',
        'e2e/**',
        'metro.config.js',
        'babel.config.js',
        'eslint.config.mjs',
        'vitest.setup.ts',
      ],
    },
    {
      files: ['**/index.ts', '**/index.tsx'],
      rules: {
        'jsdoc/require-jsdoc': 'off',
      },
    },
  ],
  { tsconfigRootDir: import.meta.dirname },
);

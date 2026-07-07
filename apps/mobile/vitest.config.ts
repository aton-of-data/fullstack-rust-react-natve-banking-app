import path from 'node:path';

import { defineConfig } from 'vitest/config';

/**
 * Vitest configuration for mobile unit and component tests.
 */
export default defineConfig({
  esbuild: {
    jsx: 'automatic',
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
    },
  },
  test: {
    globals: true,
    environment: 'node',
    setupFiles: ['./vitest.setup.ts'],
    include: ['src/**/*.test.ts', 'src/**/*.test.tsx'],
    coverage: {
      provider: 'v8',
      include: [
        'src/features/**',
        'src/shared/lib/**',
        'src/services/**',
        'src/pages/**',
        'src/shared/ui/organisms/**',
      ],
      exclude: ['**/index.ts', '**/*.test.ts', '**/*.test.tsx', 'src/test/**'],
      thresholds: {
        lines: 90,
        functions: 86,
        branches: 85,
        statements: 90,
      },
    },
  },
});

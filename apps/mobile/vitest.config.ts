import path from 'node:path';

import { defineConfig } from 'vitest/config';

/**
 * Vitest configuration for mobile unit tests (reducers and pure utilities).
 */
export default defineConfig({
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
    },
  },
  test: {
    globals: true,
    environment: 'node',
    include: ['src/**/*.test.ts'],
    coverage: {
      provider: 'v8',
      include: [
        'src/features/**',
        'src/shared/lib/**',
        'src/services/**',
      ],
    },
  },
});

/* eslint-disable @typescript-eslint/no-explicit-any -- test store helper uses loose typing */
import { configureStore } from '@reduxjs/toolkit';

import { authReducer } from '@/features/auth';
import { transferFormReducer } from '@/features/transfer-form';
import { transferSubmissionReducer } from '@/features/transfer-submission';
import { uiReducer } from '@/features/ui';
import { baseApi } from '@/services/baseApi';

/**
 * Creates a Redux store for component tests.
 *
 * @param preloadedState Optional partial initial state.
 * @returns Configured test store.
 */
export function createTestStore(preloadedState?: Record<string, unknown>) {
  const options: {
    reducer: Record<string, unknown>;
    middleware: (getDefaultMiddleware: any) => any;
    preloadedState?: Record<string, unknown>;
  } = {
    reducer: {
      auth: authReducer,
      transferForm: transferFormReducer,
      transferSubmission: transferSubmissionReducer,
      ui: uiReducer,
      [baseApi.reducerPath]: baseApi.reducer,
    },
    middleware: (getDefaultMiddleware: any) => getDefaultMiddleware().concat(baseApi.middleware),
  };

  if (preloadedState) {
    options.preloadedState = preloadedState;
  }

  return configureStore(options as any);
}

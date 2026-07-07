import { configureStore, type PreloadedState } from '@reduxjs/toolkit';

import { authReducer } from '@/features/auth';
import { transferFormReducer } from '@/features/transfer-form';
import { transferSubmissionReducer } from '@/features/transfer-submission';
import { uiReducer } from '@/features/ui';
import { baseApi } from '@/services/baseApi';
import type { RootState } from '@/store';

/**
 * Creates a Redux store for component tests.
 *
 * @param preloadedState Optional partial initial state.
 * @returns Configured test store.
 */
export function createTestStore(preloadedState?: PreloadedState<RootState>) {
  return configureStore({
    reducer: {
      auth: authReducer,
      transferForm: transferFormReducer,
      transferSubmission: transferSubmissionReducer,
      ui: uiReducer,
      [baseApi.reducerPath]: baseApi.reducer,
    },
    middleware: (getDefaultMiddleware) => getDefaultMiddleware().concat(baseApi.middleware),
    preloadedState,
  });
}

import { configureStore } from '@reduxjs/toolkit';

import { authReducer } from '@/features/auth';
import { transferFormReducer } from '@/features/transfer-form';
import { transferSubmissionReducer } from '@/features/transfer-submission';
import { uiReducer } from '@/features/ui';
import { baseApi } from '@/services/baseApi';

import { bootstrapAuth, listenerMiddleware } from './listenerMiddleware';

/**
 * Configured Redux store with RTK Query and listener middleware.
 */
export const store = configureStore({
  reducer: {
    auth: authReducer,
    transferForm: transferFormReducer,
    transferSubmission: transferSubmissionReducer,
    ui: uiReducer,
    [baseApi.reducerPath]: baseApi.reducer,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware().prepend(listenerMiddleware.middleware).concat(baseApi.middleware),
});

/** Root state type derived from the store. */
export type RootState = ReturnType<typeof store.getState>;

/** App dispatch type with thunk support. */
export type AppDispatch = typeof store.dispatch;

store.dispatch(bootstrapAuth());

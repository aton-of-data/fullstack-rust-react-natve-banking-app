import { useDispatch, useSelector, useStore, type TypedUseSelectorHook } from 'react-redux';

import type { AppDispatch, RootState } from './index';
import type { store } from './index';

/**
 * Typed dispatch hook for the Ficus store.
 *
 * @returns Typed Redux dispatch.
 */
export function useAppDispatch(): AppDispatch {
  return useDispatch<AppDispatch>();
}

/**
 * Typed selector hook for the Ficus store.
 */
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;

/**
 * Typed store hook for reading live state outside selectors (double-submit guards).
 *
 * @returns The configured Redux store.
 */
export function useAppStore(): typeof store {
  return useStore<RootState>() as typeof store;
}

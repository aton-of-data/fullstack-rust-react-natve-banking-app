import { useDispatch, useSelector, type TypedUseSelectorHook } from 'react-redux';

import type { AppDispatch, RootState } from './index';

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

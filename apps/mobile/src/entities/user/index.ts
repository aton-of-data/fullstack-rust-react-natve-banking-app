export type { MeResponse, UserSearchItem } from "@ficus/contracts";

/**
 * Domain alias for a Ficus user entity.
 */
export type UserEntity = {
  /** User identifier. */
  id: string;
  /** Username handle. */
  username: string;
};

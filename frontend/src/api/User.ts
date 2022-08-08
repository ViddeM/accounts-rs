import { AuthorityLevel } from "./AuthorityLevel";

export interface User {
  firstName: string;
  lastName: string;
  authority: AuthorityLevel;
}

import { AuthorityLevel } from "./AuthorityLevel";

export interface Me {
  firstName: string;
  lastName: string;
  email?: string;
  authority: AuthorityLevel;
}

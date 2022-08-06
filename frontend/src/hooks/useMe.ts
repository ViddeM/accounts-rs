import React, { useContext } from "react";
import { Me } from "../api/Me";

export interface AuthContext {
  me?: Me;
}

export const AuthContext = React.createContext<AuthContext>({});

export interface Auth {
  me?: Me;
}

export const useMe = (): Auth => {
  const { me } = useContext(AuthContext);

  return {
    me: me,
  };
};

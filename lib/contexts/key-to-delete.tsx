import { createContext } from "react";

export type KeyToDelete = {
  key: string | undefined;
  setKey: (value: string | undefined) => void;
};

export const KeyToDeleteContext = createContext<KeyToDelete | undefined>(
  undefined,
);

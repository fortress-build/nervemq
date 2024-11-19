import { create } from "zustand";

export type AdminSession = {
  email: string;
};

export type GlobalState = {
  session: AdminSession | undefined;
};

export const useGlobalState = create<GlobalState>((set) => ({
  session: undefined,

  setSession: (session: AdminSession) => set({ session }),
}));

export function useSession(): AdminSession | undefined {
  return useGlobalState((state) => state.session);
}

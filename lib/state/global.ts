import { create } from "zustand";

export enum Role {
  Admin = "admin",
  User = "user"
}

export type AdminSession = {
  email: string;
  role: Role;
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

export function useIsAdmin(): boolean {
  return useGlobalState((state) => state.session?.role === Role.Admin)
}

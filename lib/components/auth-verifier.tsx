"use client";

import { useVerifyUser } from "@/lib/hooks/use-verify";

export function AuthVerifier() {
  useVerifyUser();
  return null; // This component doesn't render anything
}

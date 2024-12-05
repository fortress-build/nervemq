"use client";

import { useVerifyUser } from "@/hooks/use-verify";

export function AuthVerifier() {
  useVerifyUser();
  return null; // This component doesn't render anything
}


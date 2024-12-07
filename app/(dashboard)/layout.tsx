import { AuthVerifier } from "@/lib/components/auth-verifier";
import Header from "@/lib/components/header";
import DashboardSidebar from "@/lib/components/sidebar";
import type React from "react";

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <>
        <AuthVerifier />
      <DashboardSidebar />

      <div className="flex flex-col w-full min-h-svh bg-background gap-2 px-4">
        <Header className="h-12" />
        <div>{children}</div>
      </div>
    </>
  );
}

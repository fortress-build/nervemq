"use client";

import { SidebarProvider } from "@/components/ui/sidebar";
import { Toaster } from "@/components/ui/sonner";

export default function Providers({
  sidebarOpen,
  children,
}: {
  sidebarOpen: boolean;
  children: React.ReactNode;
}) {
  return (
    <SidebarProvider defaultOpen={sidebarOpen}>
      <Toaster closeButton={true} richColors={true} />
      {children}
    </SidebarProvider>
  );
}

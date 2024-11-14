"use client";

import { SidebarProvider } from "@/components/ui/sidebar";
import { Toaster } from "@/components/ui/sonner";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const queryClient = new QueryClient();

export default function Providers({
  sidebarOpen,
  children,
}: {
  sidebarOpen: boolean;
  children: React.ReactNode;
}) {
  return (
    <SidebarProvider defaultOpen={sidebarOpen}>
      <QueryClientProvider client={queryClient}>
        <Toaster closeButton={true} richColors={true} />
        {children}
      </QueryClientProvider>
    </SidebarProvider>
  );
}

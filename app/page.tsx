"use client";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarProvider,
} from "@/components/ui/sidebar";

export default function Home() {
  return (
    <SidebarProvider>
      <Sidebar>
        <SidebarHeader>implement me</SidebarHeader>
        <SidebarContent className="flex items-center justify-center">
          implement me
        </SidebarContent>
        <SidebarFooter>implement me</SidebarFooter>
      </Sidebar>
      <main className="w-full flex items-center justify-center">
        implement me
      </main>
    </SidebarProvider>
  );
}

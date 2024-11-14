"use client";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import { Activity, Brackets, ListEnd, Logs } from "lucide-react";

const routes = [
  {
    title: "Queues",
    url: "/",
    icon: Logs,
  },
  {
    title: "Namespaces",
    url: "/",
    icon: Activity,
  },
  {
    title: "Brackets",
    url: "/",
    icon: Brackets,
  },
  {
    title: "List End",
    url: "/",
    icon: ListEnd,
  },
];

export default function Home() {
  return (
    <SidebarProvider>
      <Sidebar collapsible="icon">
        <SidebarContent className="flex items-center justify-center">
          <SidebarGroup>
            <SidebarGroupLabel>test</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {routes.map((item) => (
                  <SidebarMenuItem key={item.title}>
                    <SidebarMenuButton asChild>
                      <a href={item.url}>
                        <item.icon />
                        {item.title}
                      </a>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
          <SidebarGroup>test</SidebarGroup>
        </SidebarContent>
        <SidebarFooter>
          <SidebarTrigger />
        </SidebarFooter>
      </Sidebar>
      <main className="w-full flex items-center justify-center">
        <SidebarTrigger />
        implement me
      </main>
    </SidebarProvider>
  );
}

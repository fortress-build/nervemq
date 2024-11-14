"use client";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarTrigger,
  useSidebar,
} from "@/components/ui/sidebar";
import { cn } from "@/lib/utils";
import { Axis3D, Braces, Logs } from "lucide-react";

const routes = [
  {
    title: "Queues",
    url: "/",
    icon: Logs,
  },
  {
    title: "Namespaces",
    url: "/namespaces",
    icon: Braces,
    // icon: Activity,
  },
];

export default function DashboardSidebar() {
  const { open } = useSidebar();

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <h1
          className={cn(
            "flex gap-1 font-semibold justify-start",
            open ? "py-2 px-1" : "py-2 pl-1",
          )}
        >
          <Axis3D />
        </h1>
      </SidebarHeader>
      <SidebarContent className="flex">
        <SidebarGroup>
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
      </SidebarContent>
      <SidebarFooter>
        <SidebarTrigger />
      </SidebarFooter>
    </Sidebar>
  );
}

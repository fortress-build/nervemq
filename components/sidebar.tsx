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
} from "@/components/ui/sidebar";
import { cn } from "@/lib/utils";
import { Axis3D, Braces, Logs } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import ThemeSelector from "./theme";

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
  const pathName = usePathname();

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader
        className={cn("flex gap-1 font-semibold justify-center p-3")}
      >
        <Axis3D />
      </SidebarHeader>
      <SidebarContent className="flex">
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {routes.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton
                    isActive={pathName.endsWith(item.url)}
                    tooltip={item.title}
                    asChild
                  >
                    <Link href={item.url}>
                      <item.icon />
                      {item.title}
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="flex flex-col gap-1">
        <ThemeSelector />
        <SidebarTrigger size={"sm"} />
      </SidebarFooter>
    </Sidebar>
  );
}

"use client";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarTrigger,
  useSidebar,
} from "@/components/ui/sidebar";
import { cn } from "@/lib/utils";
import {
  Axis3D,
  Braces,
  Key,
  Logs,
  PanelLeftClose,
  PanelLeftOpen,
  Plus,
} from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import ThemeSelector from "./theme";
import { Tooltip, TooltipContent } from "./ui/tooltip";
import { TooltipTrigger } from "@radix-ui/react-tooltip";
import { useState, type MouseEventHandler } from "react";
import CreateQueue from "./create-queue";
import CreateNamespace from "./create-namespace";

function SidebarItem({
  title,
  url,
  icon: Icon,
  isActive,
  onClick,
}: {
  title: string;
  url: string;
  isActive: boolean;
  onClick?: MouseEventHandler<HTMLButtonElement>;
  icon: JSX.ElementType;
}) {
  return (
    <SidebarMenuItem key={title}>
      <SidebarMenuButton
        className="transition-all duration-200"
        isActive={isActive}
        tooltip={title}
        asChild
      >
        <Link href={url}>
          <Icon />
          {title}
        </Link>
      </SidebarMenuButton>
      <Tooltip>
        <TooltipContent side="right">Create</TooltipContent>
        <SidebarMenuAction asChild onClick={onClick}>
          <TooltipTrigger>
            <Plus />
          </TooltipTrigger>
        </SidebarMenuAction>
      </Tooltip>
    </SidebarMenuItem>
  );
}

export default function DashboardSidebar() {
  const pathName = usePathname();
  const { open } = useSidebar();

  type Mode = "normal" | "create-queue" | "create-namespace";

  const [mode, setMode] = useState<Mode>("normal");

  return (
    <Sidebar collapsible="icon">
      <CreateQueue
        open={mode === "create-queue"}
        close={() => setMode("normal")}
      />
      <CreateNamespace
        open={mode === "create-namespace"}
        close={() => setMode("normal")}
      />
      <SidebarHeader
        className={cn("flex gap-1 font-semibold justify-center p-3")}
      >
        <Axis3D />
      </SidebarHeader>
      <SidebarContent className="flex">
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarItem
                title="Queues"
                url="/queues"
                icon={Logs}
                isActive={pathName.endsWith("/")}
                onClick={() => setMode("create-queue")}
              />
              <SidebarItem
                title="Namespaces"
                url="/namespaces"
                icon={Braces}
                isActive={pathName.endsWith("/namespaces")}
                onClick={() => setMode("create-namespace")}
              />
              <SidebarItem
                title="API Keys"
                url="/api-keys"
                icon={Key}
                isActive={pathName.endsWith("/api-keys")}
              />
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="flex flex-col gap-1">
        <ThemeSelector />
        <SidebarTrigger
          size={"sm"}
          className="hover:bg-sidebar-accent cursor-pointer"
        >
          {open ? <PanelLeftClose /> : <PanelLeftOpen />}
        </SidebarTrigger>
      </SidebarFooter>
    </Sidebar>
  );
}

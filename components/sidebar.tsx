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
  Users,
} from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import ThemeSelector from "./theme";
import { Tooltip, TooltipContent } from "./ui/tooltip";
import { TooltipTrigger } from "@radix-ui/react-tooltip";
import { useState, type MouseEventHandler } from "react";
import CreateQueue from "./create-queue";
import CreateNamespace from "./create-namespace";
import CreateApiKey from "./create-api-key";
import CreateUser from "./create-user";
import { useIsAdmin } from "@/lib/state/global";
import { DialogTitle } from "./ui/dialog";

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
  const { setOpenMobile, isMobile } = useSidebar();

  return (
    <SidebarMenuItem key={title}>
      <SidebarMenuButton
        className="transition-all duration-200"
        isActive={isActive}
        tooltip={title}
        asChild
      >
        <Link
          onClick={() => {
            if (isMobile) {
              setOpenMobile(false);
            }
          }}
          href={url}
          className="whitespace-nowrap"
        >
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
  const { open, isMobile } = useSidebar();

  type Mode =
    | "normal"
    | "create-queue"
    | "create-namespace"
    | "create-api-key"
    | "create-user";

  const [mode, setMode] = useState<Mode>("normal");

  return (
    <Sidebar collapsible="icon" className="max-sm:w-full">
      <CreateQueue
        open={mode === "create-queue"}
        close={() => setMode("normal")}
      />
      <CreateNamespace
        open={mode === "create-namespace"}
        close={() => setMode("normal")}
      />
      <CreateApiKey
        open={mode === "create-api-key"}
        close={() => setMode("normal")}
      />
      <CreateUser
        open={mode === "create-user"}
        close={() => setMode("normal")}
      />

      <SidebarHeader
        className={cn("flex gap-1 font-semibold justify-center p-3")}
      >
        <Axis3D />
      </SidebarHeader>
      <SidebarContent className="flex">
        {isMobile ? <DialogTitle className="hidden">Menu</DialogTitle> : null}
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
                onClick={() => setMode("create-api-key")}
              />
              {useIsAdmin() && (
                <SidebarItem
                  title="Admin"
                  url="/admin"
                  icon={Users}
                  isActive={pathName.endsWith("/admin")}
                  onClick={() => setMode("create-user")}
                />
              )}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="flex flex-col gap-1">
        <ThemeSelector />
        <SidebarTrigger
          size={"sm"}
          className={
            isMobile ? "hidden" : "hover:bg-sidebar-accent cursor-pointer"
          }
        >
          {open ? <PanelLeftClose /> : <PanelLeftOpen />}
        </SidebarTrigger>
      </SidebarFooter>
    </Sidebar>
  );
}

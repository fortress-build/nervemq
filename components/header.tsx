"use client";

import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { useSession } from "@/lib/state/global";
import { cn } from "@/lib/utils";
import { Menu, Slash, User } from "lucide-react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import {
  DropdownMenu,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuContent,
} from "./ui/dropdown-menu";
import { SERVER_ENDPOINT } from "@/app/globals";
import { Button } from "./ui/button";
import { useSidebar } from "./ui/sidebar";

export default function Header({ className }: { className?: string }) {
  const pathName = usePathname();
  const session = useSession();
  const router = useRouter();

  let route: { label: string; href: string }[];
  if (pathName === "/") {
    route = [
      {
        label: "Queues",
        href: "/",
      },
    ];
  } else {
    const cleanPath = pathName.split('?')[0];
    const segments = cleanPath.split("/").filter((s) => s.length > 0);
    route = segments.map((s, i) => ({
      label: s.split("-")[0]
        .charAt(0) + s.split("-")[0].slice(1),
      href: `/${segments.slice(0, i + 1).join("/")}`,
    }));
  }

  const { isMobile, setOpenMobile, openMobile } = useSidebar();

  return (
    <header
      className={cn(className, "flex flex-row items-center justify-between")}
    >
      <div className="flex flex-row items-center gap-2">
        <Button
          onClick={() => setOpenMobile(!openMobile)}
          size={"sm"}
          variant={"ghost"}
          className={cn(
            "p-1.5 h-min",
            // display: none on desktop
            isMobile ? "" : "hidden",
          )}
        >
          <Menu />
        </Button>
        <Breadcrumb>
          <BreadcrumbList>
            {route.flatMap((value, i) => [
              <BreadcrumbSeparator
                className={i > 0 ? "" : "hidden"}
                key={`sep-${i.toString()}`}
              >
                <Slash />
              </BreadcrumbSeparator>,
              <BreadcrumbItem key={value.href}>
                <BreadcrumbLink
                  className={cn(
                    "text-primary text-lg",
                    i === 0 ? "font-semibold" : "font-medium",
                  )}
                  asChild
                >
                  <Link href={value.href}>{value.label}</Link>
                </BreadcrumbLink>
              </BreadcrumbItem>,
            ])}
          </BreadcrumbList>
        </Breadcrumb>
      </div>

      <DropdownMenu>
        <DropdownMenuTrigger
          className={cn("flex flex-row gap-2 text-sm items-center group")}
        >
          {session?.email ?? "Anonymous"}
          <User className="py-0.5 px-1.5 w-7 h-7 rounded-md transition-all group-hover:bg-accent" />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuLabel>NerveMQ</DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="cursor-pointer"
            onClick={() => {
              fetch(`${SERVER_ENDPOINT}/auth/logout`, {
                method: "POST",
                credentials: "include",
              }).then(() => {
                router.replace("/login");
              });
            }}
          >
            Log out
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </header>
  );
}

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
import { Slash, User } from "lucide-react";
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
    const segments = pathName.split("/").filter((s) => s.length > 0);
    route = segments.map((s, i) => ({
      label: s.charAt(0).toUpperCase() + s.slice(1),
      href: `/${segments.slice(0, i + 1).join("/")}`,
    }));
  }

  return (
    <header
      className={cn(className, "flex flex-row items-center justify-between")}
    >
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
      <DropdownMenu>
        <DropdownMenuTrigger className="flex flex-row gap-2 text-sm items-center">
          {session?.email ?? "Anonymous"}
          <User />
        </DropdownMenuTrigger>
        <DropdownMenuContent>
          <DropdownMenuLabel>NerveMQ</DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem
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

"use client";

import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { cn } from "@/lib/utils";
import { Slash } from "lucide-react";
import { usePathname } from "next/navigation";

export default function Header({ className }: { className?: string }) {
  const pathName = usePathname();

  let route: string[];
  if (pathName === "/") {
    route = ["Queues"];
  } else {
    route = pathName
      .split("/")
      .filter((s) => s.length > 0)
      .map((s) => s.charAt(0).toUpperCase() + s.slice(1));
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
            <BreadcrumbItem key={value}>
              <BreadcrumbLink
                className={cn(
                  "text-primary text-lg",
                  i === 0 ? "font-semibold" : "font-medium",
                )}
              >
                {value}
              </BreadcrumbLink>
            </BreadcrumbItem>,
          ])}
        </BreadcrumbList>
      </Breadcrumb>
    </header>
  );
}

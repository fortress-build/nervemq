"use client";
import useClickOutside from "@/lib/hooks/use-click-outside";
import { cn } from "@/lib/utils";
import { Computer, Moon, Sun } from "lucide-react";
import { useRef, useState, useEffect, type JSX } from "react";
import { sidebarMenuButtonVariants } from "./ui/sidebar";

import React from "react";
import { Button } from "./ui/button";
import { useTheme, type UseThemeProps } from "next-themes";

enum Theme {
  Light = "light",
  Dark = "dark",
  System = "system",
}

type ThemeProps = {
  icon: JSX.ElementType;
};

const themes: { [K in Theme]: ThemeProps } = {
  [Theme.Light]: {
    icon: Sun,
  },
  [Theme.Dark]: {
    icon: Moon,
  },
  [Theme.System]: {
    icon: Computer,
  },
};

export default function ThemeSelector() {
  const formContainerRef = useRef<HTMLDivElement | null>(null);
  const [isOpen, setIsOpen] = useState(false);
  const { theme = "light", setTheme } = useTheme() as UseThemeProps & {
    theme: Theme;
  };

  useClickOutside(formContainerRef, () => {
    setIsOpen(false);
  });

  const onValueChange = (value: string) => {
    setTheme(value);
    setIsOpen(false);
  };

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setIsOpen(false);
      }
    };

    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  return (
    <div className="relative">
      <Button
        variant={"ghost"}
        key="button"
        className={cn(
          sidebarMenuButtonVariants({ variant: "default", size: "default" }),
          "flex h-8 w-8 justify-start !rounded-md",
        )}
        onClick={() => setIsOpen(true)}
      >
        <span className="flex items-center justify-center">
          {theme === "system" ? (
            <Computer />
          ) : theme === "light" ? (
            <Sun />
          ) : (
            <Moon />
          )}
        </span>
      </Button>

      {isOpen && (
        <div
          ref={formContainerRef}
          className={cn(
            "absolute rounded-md outline-border left-0 bottom-0 overflow-hidden outline",
            "flex flex-row h-8 bg-background",
          )}
        >
          {Object.entries(themes)
            .sort((a, b) => {
              if (theme === a[0]) {
                return -1;
              }
              if (theme === b[0]) {
                return 1;
              }
              return 0;
            })
            .map(([name, props]) => (
              <Button
                key={name}
                variant={"ghost"}
                onClick={() => onValueChange(name)}
                className={cn(
                  "text-primary cursor-pointer h-full py-0 flex flex-col justify-center",
                  "px-2",
                )}
              >
                <props.icon />
              </Button>
            ))}
        </div>
      )}
    </div>
  );
}

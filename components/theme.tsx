"use client";
import useClickOutside from "@/hooks/use-click-outside";
import { cn } from "@/lib/utils";
import { AnimatePresence, MotionConfig, motion } from "framer-motion";
import { Computer, Moon, Sun } from "lucide-react";
import { useRef, useState, useEffect, useId } from "react";
import { SidebarMenuButton } from "./ui/sidebar";

import React from "react";
import { Button } from "./ui/button";
import { useTheme, type UseThemeProps } from "next-themes";

const TRANSITION = {
  type: "spring",
  bounce: 0.05,
  duration: 0.3,
};

type Theme = "light" | "dark" | "system";

const themes: Record<Theme, { icon: JSX.ElementType }> = {
  light: {
    icon: Sun,
  },
  dark: {
    icon: Moon,
  },
  system: {
    icon: Computer,
  },
};

export default function ThemeSelector() {
  const uniqueId = useId();
  const formContainerRef = useRef<HTMLDivElement>(null);
  const [isOpen, setIsOpen] = useState(false);
  const { theme, setTheme } = useTheme() as UseThemeProps & {
    theme: Theme;
  };

  const onValueChange = (value: string) => {
    setTheme(value);
    setIsOpen(false);
  };

  useClickOutside(formContainerRef, () => {
    setIsOpen(false);
  });

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

  const currentTheme = themes[theme ?? "system"];

  return (
    <MotionConfig transition={TRANSITION}>
      <div className="relative">
        <motion.button
          key="button"
          layoutId={`popover-${uniqueId}`}
          className={cn("flex h-8 w-8 justify-start")}
          onClick={() => setIsOpen(true)}
        >
          <motion.span
            className="flex items-center justify-center"
            layoutId={`popover-label-${uniqueId}`}
          >
            <SidebarMenuButton asChild className="text-current w-8 h-8">
              {<currentTheme.icon />}
            </SidebarMenuButton>
          </motion.span>
        </motion.button>

        <AnimatePresence>
          {isOpen && (
            <motion.div
              ref={formContainerRef}
              layoutId={`popover-${uniqueId}`}
              className={cn(
                "absolute rounded-md border-border left-0 bottom-0 overflow-hidden border",
                "flex flex-row h-8 bg-background",
              )}
              style={{
                borderRadius: 8,
              }}
            >
              {Object.entries(themes)
                .toSorted((a, b) => {
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
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </MotionConfig>
  );
}

"use client";
import useClickOutside from "@/hooks/use-click-outside";
import { cn } from "@/lib/utils";
import { AnimatePresence, MotionConfig, motion } from "framer-motion";
import { Computer, Moon, Sun } from "lucide-react";
import { useRef, useState, useEffect, useId, useCallback } from "react";
import { SidebarMenuButton } from "./ui/sidebar";

import React from "react";
import { Button } from "./ui/button";

const TRANSITION = {
  type: "spring",
  bounce: 0.05,
  duration: 0.3,
};

export default function ThemeSelector() {
  const uniqueId = useId();
  const formContainerRef = useRef<HTMLDivElement>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [theme, setTheme] = useState("system");

  const openMenu = () => {
    setIsOpen(true);
  };

  const closeMenu = useCallback(() => {
    setIsOpen(false);
  }, []);

  const onValueChange = (value: string) => {
    setTheme(value);
    setIsOpen(false);
  };

  useClickOutside(formContainerRef, () => {
    closeMenu();
  });

  useEffect(() => {
    if (window.matchMedia === undefined) {
      return;
    }

    const match = window.matchMedia("(prefers-color-scheme: dark)");

    const updateTheme = () => {
      switch (theme) {
        case "dark":
          document.body.classList.add("dark");
          break;
        case "light":
          document.body.classList.remove("dark");
          break;
        default:
          if (match.matches) {
            document.body.classList.add("dark");
          } else {
            document.body.classList.remove("dark");
          }
          break;
      }
    };

    updateTheme();

    match.addEventListener("change", updateTheme);

    return () => {
      match.removeEventListener("change", updateTheme);
    };
  }, [theme]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        closeMenu();
      }
    };

    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [closeMenu]);

  return (
    <MotionConfig transition={TRANSITION}>
      <div className="relative">
        <motion.button
          key="button"
          layoutId={`popover-${uniqueId}`}
          className={cn("flex h-8 w-7 justify-start")}
          onClick={openMenu}
        >
          <motion.span
            className="flex items-center justify-center"
            layoutId={`popover-label-${uniqueId}`}
          >
            <SidebarMenuButton
              asChild
              className="text-muted-foreground w-7 h-7 p-1.5"
            >
              {
                {
                  system: <Computer />,
                  dark: <Moon />,
                  light: <Sun />,
                }[theme]
              }
            </SidebarMenuButton>
          </motion.span>
        </motion.button>

        <AnimatePresence>
          {isOpen && (
            <motion.div
              ref={formContainerRef}
              layoutId={`popover-${uniqueId}`}
              className={cn(
                "absolute rounded-md border-border min-w-28 left-0 bottom-0 overflow-hidden border",
                "flex flex-row h-8 bg-background",
              )}
              style={{
                borderRadius: 8,
              }}
            >
              <Button
                variant={"ghost"}
                onClick={() => onValueChange("dark")}
                className="cursor-pointer h-full py-0 flex flex-col justify-center"
              >
                <Moon />
              </Button>
              <Button
                variant={"ghost"}
                onClick={() => onValueChange("light")}
                className="cursor-pointer h-full py-0 flex flex-col justify-center"
              >
                <Sun />
              </Button>
              <Button
                variant={"ghost"}
                onClick={() => onValueChange("system")}
                className="cursor-pointer h-full py-0 flex flex-col justify-center"
              >
                <Computer />
              </Button>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </MotionConfig>
  );
}

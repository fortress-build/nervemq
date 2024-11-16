import type { Metadata } from "next";
import "./globals.css";
import { cookies } from "next/headers";
import Providers from "./providers";
import DashboardSidebar from "@/components/sidebar";
import Header from "@/components/header";

export const metadata: Metadata = {
  title: "Creek UI",
  description: "Creek admin panel",
};

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const cookieStore = await cookies();
  const defaultOpen = cookieStore.get("sidebar:state")?.value === "true";

  return (
    <html lang="en" suppressHydrationWarning>
      <body
      // className={
      //   window.matchMedia("(prefers-color-scheme: dark)").matches
      //     ? "dark"
      //     : ""
      // }
      >
        <Providers sidebarOpen={defaultOpen}>
          <DashboardSidebar />

          <div className="flex flex-col w-full min-h-svh bg-background gap-2 px-4">
            <Header className="h-12" />
            <main>{children}</main>
          </div>
        </Providers>
      </body>
    </html>
  );
}

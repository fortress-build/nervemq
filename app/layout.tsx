import type { Metadata } from "next";
import "./globals.css";
import { cookies } from "next/headers";
import Providers from "./providers";
import DashboardSidebar from "@/components/sidebar";

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
    <html lang="en">
      <body>
        <Providers sidebarOpen={defaultOpen}>
          <DashboardSidebar />

          <main className="w-full min-h-svh bg-background">{children}</main>
        </Providers>
      </body>
    </html>
  );
}

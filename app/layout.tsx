import type { Metadata } from "next";
import "./globals.css";
import { cookies } from "next/headers";
import Providers from "./providers";
import { AuthVerifier } from "@/components/auth-verifier";

export const metadata: Metadata = {
  title: "NerveMQ UI",
  description: "NerveMQ admin panel",
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
      <body>
        <AuthVerifier />
        <Providers sidebarOpen={defaultOpen}>{children}</Providers>
      </body>
    </html>
  );
}

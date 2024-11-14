import type { Metadata } from "next";
import "./globals.css";
import { cookies } from "next/headers";
import Providers from "./providers";

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
        <Providers sidebarOpen={defaultOpen}>{children}</Providers>
      </body>
    </html>
  );
}

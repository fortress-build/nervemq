import Header from "@/components/header";
import DashboardSidebar from "@/components/sidebar";

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <>
      <DashboardSidebar />

      <div className="flex flex-col w-full min-h-svh bg-background gap-2 px-4">
        <Header className="h-12" />
        <div>{children}</div>
      </div>
    </>
  );
}

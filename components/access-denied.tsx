"use client";

import { useRouter } from "next/navigation";
import { Button } from "./ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";

export default function AccessDenied({
  returnTo,
}: {
  returnTo: {
    name: string;
    href: string;
  };
}) {
  const router = useRouter();

  return (
    <div className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center">
      <Card className="w-[400px] border">
        <CardHeader className="text-center">
          <CardTitle>Access Denied</CardTitle>
        </CardHeader>
        <CardContent className="text-center">
          <p className="mb-4">
            You don&apos;t have permission to view this page.
          </p>
          <Button onClick={() => router.push(returnTo.href)}>
            Return to {returnTo.name}
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}

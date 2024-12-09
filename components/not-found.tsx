"use client";

import { useRouter } from "next/navigation";
import { Button } from "./ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";

export default function NotFound({
  resource,
  returnTo,
}: {
  resource: string;
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
          <CardTitle>Not Found</CardTitle>
        </CardHeader>
        <CardContent className="text-center">
          <p className="mb-4">
            The {resource} you are looking for does not exist.
          </p>
          <Button onClick={() => router.replace(returnTo.href)}>
            Return to {returnTo.name}
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}

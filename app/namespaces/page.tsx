"use client";
import { listNamespaces } from "@/actions/api";
import { NamespaceTable } from "@/components/namespaces/table";
import CreateNamespace from "@/components/create-namespace";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Button } from "@/components/ui/button";

export default function Namespaces() {
  const [isOpen, setIsOpen] = useState(false);
  const { data, isLoading, error } = useQuery({
    queryKey: ["list-namespaces"],
    queryFn: listNamespaces,
  });

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error loading namespaces</div>;

  return (
    <div className="h-full flex flex-col gap-4">
      <NamespaceTable data={data || []} />
      <div className="flex justify-end px-4">
        <Button onClick={() => setIsOpen(true)}>Create Namespace</Button>
      </div>
      <CreateNamespace open={isOpen} close={() => setIsOpen(false)} />
    </div>
  );
}

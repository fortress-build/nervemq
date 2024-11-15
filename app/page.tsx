"use client";

import { listQueues } from "@/actions/api";
import { useQuery } from "@tanstack/react-query";

import { columns } from "@/components/queues/table";
import { DataTable } from "@/components/data-table";
import CreateQueue from "@/components/create-queue";
import { Button } from "@/components/ui/button";
import { useState } from "react";

export default function Queues() {
  const { data = [], isLoading } = useQuery({
    queryFn: () => listQueues(),
    queryKey: ["queues"],
  });
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="flex px-4">
      <DataTable
        className="w-full"
        columns={columns}
        data={data}
        isLoading={isLoading}
      />

      <div className="flex justify-end px-4">
        <Button onClick={() => setIsOpen(true)}>Create Queue</Button>
      </div>
      <CreateQueue open={isOpen} close={() => setIsOpen(false)} />
    </div>
  );
}

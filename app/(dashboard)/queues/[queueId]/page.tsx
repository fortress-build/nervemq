"use client";
import ClientList from "../list";
import { useQuery } from "@tanstack/react-query";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Spinner } from "@nextui-org/spinner";
import type { QueueStatistics } from "@/components/queues/table";
import { listQueues } from "@/actions/api";
import { useParams } from "next/navigation";
import { QueueSettings } from "@/components/queue-settings";

export default function QueuePage() {
  const { queueId }: { queueId: string } = useParams();

  const { data: queue, isLoading } = useQuery({
    queryKey: ["queues"],
    queryFn: () => listQueues(),
    refetchInterval: 30000, // Refetch every 30 seconds
    select: (data: Map<string, QueueStatistics>) => data.get(queueId),
  });

  return (
    <>
      <div className="grid gap-4">
        {/* Queue Status Section */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle>Queue Status</CardTitle>
            <QueueSettings queue={queue} />
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-3 gap-4">
              <div>
                <p className="text-gray-600">Pending</p>
                <p className="text-2xl font-medium">5</p>
              </div>
              <div>
                <p className="text-gray-600">Delivered</p>
                <p className="text-2xl font-medium">5</p>
              </div>
              <div>
                <p className="text-gray-600">Failed</p>
                <p className="text-2xl font-medium">5</p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Metrics Section */}
        <Card>
          <CardHeader>
            <CardTitle>Metrics</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-3 gap-4">
              {isLoading ? (
                <div className="col-span-2 md:col-span-4 flex items-center justify-center">
                  <Spinner />
                </div>
              ) : queue ? (
                <>
                  <div>
                    <p className="text-gray-600">Daily Messages (avg)</p>
                    <p className="text-2xl font-medium">
                      {(queue.queue_operations_total ?? 0).toFixed(2)}
                    </p>
                  </div>
                  <div>
                    <p className="text-gray-600">Message Size (avg)</p>
                    <p className="text-2xl font-medium">
                      {(queue.avgSizeBytes ?? 0).toFixed(2)} bytes
                    </p>
                  </div>
                  <div>
                    <p className="text-gray-600">Error Rate</p>
                    <p className="text-2xl font-medium">{(0).toFixed(2)}%</p>
                  </div>
                </>
              ) : (
                <div>Failed to load metrics</div>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Current Queue Items */}
        <Card>
          <CardHeader>
            <CardTitle>Messages</CardTitle>
          </CardHeader>
          <CardContent>
            <ClientList />
          </CardContent>
        </Card>
      </div>
    </>
  );
}

"use client";
import ClientList from "@/app/(dashboard)/queues/list";
import { useQuery } from "@tanstack/react-query";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import type { QueueStatistics } from "@/components/queues/table";
import { fetchQueue } from "@/actions/api";
import { useParams, useRouter } from "next/navigation";
import { QueueSettings } from "@/components/queue-settings";
import { Button } from "@/components/ui/button";

export default function QueuePage() {
  const router = useRouter();
  const { queueId }: { queueId: [string, string] } = useParams();

  if (queueId.length < 2) {
    router.replace("/queues");
  }

  const [namespace, name] = queueId;

  const { data: queue, error } = useQuery<QueueStatistics, Error>({
    queryKey: ["queues", name, namespace],
    queryFn: () => {
      if (!name || !namespace) {
        throw new Error("Invalid queue ID");
      }
      return fetchQueue(name, namespace) as Promise<QueueStatistics>;
    },
    refetchInterval: 30000,
  });

  if (error instanceof Error && "status" in error && error.status === 403) {
    return (
      <div className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center">
        <Card className="w-[400px] border">
          <CardHeader className="text-center">
            <CardTitle>Access Denied</CardTitle>
          </CardHeader>
          <CardContent className="text-center">
            <p className="mb-4">
              You don&apos;t have permission to view this queue.
            </p>
            <Button onClick={() => router.push("/queues")}>
              Return to Queues
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

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
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
              <div>
                <p className="text-gray-600 break-words">Pending</p>
                <p className="text-2xl font-medium">{queue?.pending ?? 0}</p>
              </div>
              <div>
                <p className="text-gray-600 break-words">Delivered</p>
                <p className="text-2xl font-medium">{queue?.delivered ?? 0}</p>
              </div>
              <div>
                <p className="text-gray-600 break-words">Failed</p>
                <p className="text-2xl font-medium">{queue?.failed ?? 0}</p>
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
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
              {queue ? (
                <>
                  <div>
                    <p className="text-gray-600 break-words">Message Size (avg)</p>
                    <p className="text-2xl font-medium overflow-hidden text-ellipsis">
                      {(queue.avgSizeBytes ?? 0).toFixed(2)} bytes
                    </p>
                  </div>
                  <div>
                    <p className="text-gray-600 break-words">Error Rate</p>
                    <p className="text-2xl font-medium">
                      {(queue.failed + queue.delivered === 0
                        ? 0
                        : queue.failed / (queue.delivered + queue.failed)
                      ).toFixed(2)}
                      %
                    </p>
                  </div>
                  <div>
                    <p className="text-gray-600 break-words">Total Operations</p>
                    <p className="text-2xl font-medium">
                      {(queue.queue_operations_total ?? 0).toFixed(2)}
                    </p>
                  </div>
                </>
              ) : null}
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

"use client";
import ClientList from "../list";
import { Line } from "react-chartjs-2";
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
} from "chart.js";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Spinner } from "@nextui-org/spinner";
import type { QueueStatistics } from "@/components/queues/table";
import { listQueues } from "@/actions/api";
import { useParams } from "next/navigation";
import { QueueSettings } from "@/components/queue-settings";

// Register ChartJS components
ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
);

// Sample data - replace with real data
const chartData = {
  labels: ["1h ago", "45m ago", "30m ago", "15m ago", "Now"],
  datasets: [
    {
      label: "Pending",
      data: [4, 3, 8, 6, 5],
      borderColor: "rgb(75, 192, 192)",
      tension: 0.1,
    },
    {
      label: "Processing",
      data: [2, 4, 3, 4, 5],
      borderColor: "rgb(54, 162, 235)",
      tension: 0.1,
    },
    {
      label: "Completed",
      data: [1, 3, 4, 4, 5],
      borderColor: "rgb(75, 192, 75)",
      tension: 0.1,
    },
    {
      label: "Failed",
      data: [1, 2, 3, 4, 5],
      borderColor: "rgb(255, 99, 132)",
      tension: 0.1,
    },
  ],
};

export default function QueuePage() {
  const { queueId }: { queueId: string } = useParams();

  const { data: queue, isLoading } = useQuery({
    queryKey: ["queues"],
    queryFn: () => listQueues(),
    refetchInterval: 30000, // Refetch every 30 seconds
    select: (data: Map<string, QueueStatistics>) => data.get(queueId),
  });

  console.log(queue);

  // Add state for visibility toggles
  const [visibleDatasets, setVisibleDatasets] = useState({
    pending: true,
    processing: true,
    completed: true,
    failed: true,
  });

  // Filter datasets based on visibility state
  const filteredChartData = {
    ...chartData,
    datasets: chartData.datasets.filter((dataset) => {
      const label = dataset.label.toLowerCase();
      return (
        label in visibleDatasets &&
        visibleDatasets[label as keyof typeof visibleDatasets]
      );
    }),
  };

  // Add toggle handler
  const toggleDataset = (datasetName: string) => {
    setVisibleDatasets((prev) => ({
      ...prev,
      [datasetName]: !prev[datasetName as keyof typeof prev],
    }));
  };

  return (
    <>
      <div className="grid gap-4">
        {/* Queue Status Section */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle>Queue Status</CardTitle>
            <QueueSettings />
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-4 gap-4">
              <div>
                <p className="text-gray-600">Pending</p>
                <p className="text-2xl font-medium">5</p>
              </div>
              <div>
                <p className="text-gray-600">Processing</p>
                <p className="text-2xl font-medium">5</p>
              </div>
              <div>
                <p className="text-gray-600">Completed</p>
                <p className="text-2xl font-medium">5</p>
              </div>
              <div>
                <p className="text-gray-600">Failed</p>
                <p className="text-2xl font-medium">5</p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Prometheus Metrics Section */}
        <Card>
          <CardHeader>
            <CardTitle>Metrics</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              {isLoading ? (
                <div className="col-span-2 md:col-span-4 flex items-center justify-center">
                  <Spinner />
                </div>
              ) : queue ? (
                <>
                  <div>
                    <p className="text-gray-600">Daily Messages (avg)</p>
                    <p className="text-2xl font-medium">
                      {queue.queue_operations_total?.[0]?.value || 0}
                    </p>
                  </div>
                  <div>
                    <p className="text-gray-600">Active Connections</p>
                    <p className="text-2xl font-medium">
                      {queue.active_connections?.[0]?.value || 0}
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

        {/* Queue History Graph */}
        <Card>
          <CardHeader>
            <CardTitle>Queue History</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex gap-2 mb-4">
              <Button
                onClick={() => toggleDataset("pending")}
                variant={visibleDatasets.pending ? "default" : "secondary"}
                style={
                  visibleDatasets.pending
                    ? { backgroundColor: "rgb(75, 192, 192)" }
                    : {}
                }
              >
                Pending
              </Button>
              <Button
                onClick={() => toggleDataset("processing")}
                variant={visibleDatasets.processing ? "default" : "secondary"}
                style={
                  visibleDatasets.processing
                    ? { backgroundColor: "rgb(54, 162, 235)" }
                    : {}
                }
              >
                Processing
              </Button>
              <Button
                onClick={() => toggleDataset("completed")}
                variant={visibleDatasets.completed ? "default" : "secondary"}
                style={
                  visibleDatasets.completed
                    ? { backgroundColor: "rgb(75, 192, 75)" }
                    : {}
                }
              >
                Completed
              </Button>
              <Button
                onClick={() => toggleDataset("failed")}
                variant={visibleDatasets.failed ? "default" : "secondary"}
                style={
                  visibleDatasets.failed
                    ? { backgroundColor: "rgb(255, 99, 132)" }
                    : {}
                }
              >
                Failed
              </Button>
            </div>
            <div className="h-[300px]">
              <Line
                data={filteredChartData}
                options={{
                  maintainAspectRatio: false,
                  scales: {
                    y: {
                      ticks: {
                        stepSize: 2, // Show labels at intervals of 2
                        maxTicksLimit: 6, // Maximum number of ticks to display
                      },
                    },
                  },
                }}
              />
            </div>
          </CardContent>
        </Card>

        {/* Current Queue Items */}
        <Card>
          <CardHeader>
            <CardTitle>Current Items</CardTitle>
          </CardHeader>
          <CardContent>
            <ClientList />
          </CardContent>
        </Card>
      </div>
    </>
  );
}

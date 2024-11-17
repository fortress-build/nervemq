import ClientList from "../list";

export default function QueuePage() {
  return (
    <div className="container mx-auto">
      <div className="grid gap-4">
        {/* Queue Status Section */}
        <section className="border rounded-lg p-4">
          <h2 className="text-xl font-medium mb-2">Queue Status</h2>
          <div className="grid grid-cols-3 gap-4">
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
        </section>

        {/* Current Queue Items */}
        <section className="border rounded-lg p-4">
          <h2 className="text-xl font-medium mb-2">Current Items</h2>
          <ClientList />
        </section>

        {/* Queue Controls */}
        <section className="border rounded-lg p-4">
          <h2 className="text-xl font-medium mb-2">Controls</h2>
          {/* Add queue control buttons */}
        </section>
      </div>
    </div>
  );
}

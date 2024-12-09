import NotFound from "@/components/not-found";

export default function PageNotFound() {
  return (
    <NotFound
      resource="page"
      returnTo={{
        name: "Queues",
        href: "/queues",
      }}
    />
  );
}

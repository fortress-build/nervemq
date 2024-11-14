"use server";
import { listNamespaces } from "@/actions/api";
import { NamespaceTable } from "@/components/namespaces/table";

export default async function Namespaces() {
  const data = await listNamespaces();
  console.log(data);
  return (
    <div className="h-full flex">
    <NamespaceTable data={data}/></div>
  );
}

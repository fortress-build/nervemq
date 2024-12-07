import type { JSX } from "react";

export function ColumnHeader(props: { label: string; icon: JSX.ElementType }) {
  return (
    <span className="flex flex-row gap-1 items-center">
      <props.icon size={16} />
      {props.label}
    </span>
  );
}

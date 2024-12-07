import { useState } from "react";
import { Button } from "@/lib/components/ui/button";
import { Input } from "@/lib/components/ui/input";
import { X } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";

export default function KeyValueForm({
  value,
  onChange,
}: {
  value: Map<string, string>;
  onChange: (value: Map<string, string>) => void;
}) {
  const [currentKey, setCurrentKey] = useState("");
  const [currentValue, setCurrentValue] = useState("");

  const addPair = () => {
    if (currentKey && currentValue) {
      if (value.has(currentKey)) {
        toast.warning("Key already exists");
        return;
      }

      const pairs = new Map([
        [currentKey, currentValue],
        ...Array.from(value.entries().filter(([key]) => key !== currentKey)),
      ]);
      onChange(pairs);

      setCurrentKey("");
      setCurrentValue("");
    }
  };

  const removePair = (key: string) => {
    value.delete(key);
    onChange(new Map(value.entries()));
  };

  return (
    <div className="w-full max-w-2xl space-y-4">
      <div className="flex space-x-4">
        <div className="flex-1 space-y-2">
          <Input
            id="key"
            value={currentKey}
            onChange={(e) => setCurrentKey(e.target.value)}
            placeholder="Key"
          />
        </div>
        <div className="flex-1 space-y-2">
          <Input
            id="value"
            value={currentValue}
            onChange={(e) => setCurrentValue(e.target.value)}
            placeholder="Value"
          />
        </div>
        <div className="flex items-end">
          <Button onClick={addPair}>Add</Button>
        </div>
      </div>

      {/* Tags don't really do anything at the moment, do we really need them? */}
      <div
        className={cn(
          "rounded-md overflow-hidden",
          value.size > 0 ? "border" : "",
        )}
      >
        {value.size > 0 ? (
          <div className="grid grid-cols-2 gap-4 px-4 py-1.5 bg-secondary font-medium">
            <div>Key</div>
            <div>Value</div>
          </div>
        ) : null}
        {value
          .entries()
          .map(([key, value], index) => (
            <div
              key={`${key}-${index.toString()}`}
              className="grid grid-cols-2 gap-4 px-4 items-center"
            >
              <div>{key}</div>
              <div className="flex items-center justify-between">
                <span>{value}</span>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => removePair(key)}
                >
                  <X className="h-4 w-4" />
                  <span className="sr-only">Remove</span>
                </Button>
              </div>
            </div>
          ))
          .toArray()}
      </div>
    </div>
  );
}

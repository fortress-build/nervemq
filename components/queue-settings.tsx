"use client";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Settings2 } from "lucide-react";

export function QueueSettings() {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button size="icon">
          <Settings2 className="h-4 w-4" />
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Queue Settings</DialogTitle>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="retries" className="text-right">
              Max Retries
            </Label>
            <Input
              id="retries"
              type="number"
              className="col-span-3"
              defaultValue={3}
            />
          </div>
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="timeout" className="text-right">
              Timeout (s)
            </Label>
            <Input
              id="timeout"
              type="number"
              className="col-span-3"
              defaultValue={30}
            />
          </div>
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="batch-size" className="text-right">
              Batch Size
            </Label>
            <Input
              id="batch-size"
              type="number"
              className="col-span-3"
              defaultValue={100}
            />
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

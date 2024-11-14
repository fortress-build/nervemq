import { Button } from "./ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "./ui/dialog";

import { type InferType, object, string } from "yup";
import { useForm } from "@tanstack/react-form";
import { yupValidator } from "@tanstack/yup-form-adapter";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { cn } from "@/lib/utils";
import { Spinner } from "@nextui-org/react";
import { createQueue } from "@/actions/api";

function isAlphaNumeric(str: string) {
  let code: number;
  let i: number;
  let len: number;

  for (i = 0, len = str.length; i < len; i++) {
    code = str.charCodeAt(i);
    if (
      !(code > 47 && code < 58) && // numeric (0-9)
      !(code > 64 && code < 91) && // upper alpha (A-Z)
      !(code > 96 && code < 123)
    ) {
      // lower alpha (a-z)
      return false;
    }
  }
  return true;
}

const createQueueSchema = object({
  name: string()
    .required()
    .max(32)
    .min(1)
    .test("name", "name should be alphanumeric", (value: string) => {
      return isAlphaNumeric(value);
    }),
});

export type CreateQueue = InferType<typeof createQueueSchema>;

export default function CreateQueue({
  open,
  close,
}: {
  open: boolean;
  close: () => void;
}) {
  const form = useForm({
    defaultValues: {
      name: "",
    },
    validatorAdapter: yupValidator(),
    validators: {
      onChange: createQueueSchema,
      onMount: createQueueSchema,
    },
    onSubmit: async ({ value: data }) => {
      console.log(data);
      await createQueue(data);
    },
  });

  return (
    <Dialog
      open={open}
      onOpenChange={(open) => {
        if (!open) {
          close();
        }
      }}
    >
      <DialogDescription>Create Queue</DialogDescription>
      <DialogContent>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            e.stopPropagation();
            void form.handleSubmit();
          }}
          className="flex flex-col gap-4"
        >
          <DialogHeader>
            <DialogTitle>Create Queue</DialogTitle>
          </DialogHeader>
          <form.Field name="name">
            {(field) => (
              <div className="flex flex-col gap-2">
                <Label htmlFor={field.name}>Name</Label>
                <Input
                  id={field.name}
                  name={field.name}
                  value={field.state.value}
                  type="text"
                  onBlur={field.handleBlur}
                  onChange={(e) => field.handleChange(e.target.value)}
                  placeholder="Name"
                  className={cn(
                    "focus-visible:outline-none focus-visible:ring-0 focus-visible:ring-offset-0",
                    "focus:border-primary focus:border transition-all",
                  )}
                />
                {field.state.meta.errors ? (
                  <span>{field.state.meta.errors.join(", ")}</span>
                ) : null}
              </div>
            )}
          </form.Field>
          <DialogFooter>
            <form.Subscribe
              selector={(state) => [state.canSubmit, state.isSubmitting]}
            >
              {([canSubmit, isSubmitting]) => (
                <Button type="submit" disabled={!canSubmit}>
                  {isSubmitting ? (
                    <>
                      <Spinner
                        color="white"
                        className="w-16 absolute self-center"
                      />
                      <p className="text-transparent">Create</p>
                    </>
                  ) : (
                    "Create"
                  )}
                </Button>
              )}
            </form.Subscribe>
            <Button variant={"secondary"}>Cancel</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

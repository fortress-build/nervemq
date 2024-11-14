import { Button } from "./ui/button";
import {
  Dialog,
  DialogClose,
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
import { createQueue } from "@/actions/api";
import { Spinner } from "@nextui-org/react";
import { Check, ChevronsUpDown } from "lucide-react";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "./ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "./ui/popover";

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
  namespace: string().required("namespace is a required field"),
});

const namespaces = [
  { label: "Default", value: "default" },
  { label: "Production", value: "prod" },
  { label: "Development", value: "dev" },
  // Add more namespaces as needed
] as const;

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
      namespace: "",
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
            <DialogDescription>
              There is no limit to the number of queues you can create.
            </DialogDescription>
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
                  <span className="text-sm text-destructive">
                    {field.state.meta.errors.join(", ")}
                  </span>
                ) : null}
              </div>
            )}
          </form.Field>
          <form.Field name="namespace">
            {(field) => (
              <div className="flex flex-col gap-2">
                <Label htmlFor={field.name}>Namespace</Label>
                <Popover>
                  <PopoverTrigger asChild>
                    <Button
                      variant="outline"
                      // biome-ignore lint/a11y/useSemanticElements: <explanation>
                      role="combobox"
                      className={cn(
                        "w-full justify-between",
                        !field.state.value && "text-muted-foreground",
                      )}
                    >
                      {field.state.value
                        ? namespaces.find(
                            (namespace) =>
                              namespace.value === field.state.value,
                          )?.label
                        : "Select namespace"}
                      <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent className="w-full p-0">
                    <Command>
                      <CommandInput placeholder="Search namespace..." />
                      <CommandList>
                        <CommandEmpty>No namespace found.</CommandEmpty>
                        <CommandGroup>
                          {namespaces.map((namespace) => (
                            <CommandItem
                              key={namespace.value}
                              value={namespace.value}
                              onSelect={(currentValue) => {
                                field.handleChange(
                                  currentValue === field.state.value
                                    ? ""
                                    : currentValue,
                                );
                              }}
                            >
                              <Check
                                className={cn(
                                  "mr-2 h-4 w-4",
                                  field.state.value === namespace.value
                                    ? "opacity-100"
                                    : "opacity-0",
                                )}
                              />
                              {namespace.label}
                            </CommandItem>
                          ))}
                        </CommandGroup>
                      </CommandList>
                    </Command>
                  </PopoverContent>
                </Popover>
                {field.state.meta.errors ? (
                  <span className="text-sm text-destructive">
                    {field.state.meta.errors.join(", ")}
                  </span>
                ) : null}
              </div>
            )}
          </form.Field>

          <DialogFooter>
            <form.Subscribe
              selector={(state) => [state.canSubmit, state.isSubmitting]}
            >
              {([canSubmit, isSubmitting]) => (
                <>
                  <Button type="submit" disabled={!canSubmit}>
                    {isSubmitting ? (
                      <>
                        {
                          <Spinner
                            className="absolute self-center"
                            size="sm"
                            color="current"
                          />
                        }
                        <p className="text-transparent">Create</p>
                      </>
                    ) : (
                      "Create"
                    )}
                  </Button>

                  <DialogClose asChild>
                    <Button variant={"secondary"} disabled={isSubmitting}>
                      Cancel
                    </Button>
                  </DialogClose>
                </>
              )}
            </form.Subscribe>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

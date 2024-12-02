import { useState, useCallback } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogTitle,
  DialogClose,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { cn, isAlphaNumeric } from "@/lib/utils";
import { type InferType, object, string } from "yup";
import { useForm } from "@tanstack/react-form";
import { yupValidator } from "@tanstack/yup-form-adapter";
import { Spinner } from "@nextui-org/react";
import { Input } from "@/components/ui/input";
import { useInvalidate } from "@/hooks/use-invalidate";
import { DialogHeader } from "./ui/dialog";
import { createAPIKey } from "@/actions/api";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Copy as CopyIcon, Info as InfoIcon } from "lucide-react";
// Add schema
export const createApiKeySchema = object({
  name: string()
    .required()
    .max(32)
    .min(1)
    .test("name", "name should be alphanumeric", (value: string) => {
      return isAlphaNumeric(value);
    }),
});

export type CreateApiKey = InferType<typeof createApiKeySchema>;

export interface APIKey {
  name: string;
  access_key: string;
  secret_key: string;
  prefix: string;
  token?: string;
}

interface CreateApiKeyProps {
  open: boolean;
  close: () => void;
  onSuccess?: (keyName: string) => void;
}

export default function CreateApiKey({
  open,
  close,
  onSuccess,
}: CreateApiKeyProps) {
  const [showKey, setShowKey] = useState(false);
  const [apiKey, setApiKey] = useState<APIKey | null>(null);
  const invalidate = useInvalidate(["apiKeys"]);

  const form = useForm({
    defaultValues: {
      name: "",
    },
    validatorAdapter: yupValidator(),
    validators: {
      onChange: createApiKeySchema,
      onMount: createApiKeySchema,
    },
    onSubmit: async ({ value: data, formApi }) => {
      await createAPIKey(data)
        .then((result) => {
          setApiKey(result);
          setShowKey(true);
          invalidate();
          if (onSuccess) {
            onSuccess(data.name);
          }
          formApi.reset();
        })
        .catch(() => {
          toast.error("Failed to create API key");
        });
    },
  });

  const downloadKey = useCallback(() => {
    if (apiKey?.token) {
      const content = [
        `Platform API Key: nervemq_${apiKey.access_key}_${apiKey.secret_key}`,
        `Access Key: ${apiKey.access_key}`,
        `Secret Key: ${apiKey.secret_key}`,
      ].join("\n");

      const blob = new Blob([content], { type: "text/plain" });
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `api-key-${apiKey.name}.txt`;
      a.click();
      window.URL.revokeObjectURL(url);
    }
  }, [apiKey]);

  return (
    <Dialog
      open={open}
      onOpenChange={(open) => {
        if (!open) {
          close();
          setShowKey(false);
          setApiKey(null);
        }
      }}
    >
      <DialogContent className="rounded-lg sm:rounded-lg">
        {!showKey ? (
          <form
            onSubmit={(e) => {
              e.preventDefault();
              e.stopPropagation();
              void form.handleSubmit();
            }}
            className="flex flex-col gap-4"
          >
            <DialogHeader>
              <DialogTitle>Create API Key</DialogTitle>
              <DialogDescription>
                Create a new API key for accessing the API.
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
                    placeholder="My API Key"
                    data-1p-ignore
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

            <DialogFooter>
              <form.Subscribe
                selector={(state) => [state.canSubmit, state.isSubmitting]}
              >
                {([canSubmit, isSubmitting]) => (
                  <div className="flex flex-col sm:flex-row gap-2">
                    <Button type="submit" disabled={!canSubmit}>
                      {isSubmitting ? (
                        <>
                          <Spinner
                            className="absolute self-center"
                            size="sm"
                            color="current"
                          />
                          <p className="text-transparent">Create</p>
                        </>
                      ) : (
                        "Create"
                      )}
                    </Button>
                    <DialogClose asChild>
                      <Button variant="secondary" disabled={isSubmitting}>
                        Cancel
                      </Button>
                    </DialogClose>
                  </div>
                )}
              </form.Subscribe>
            </DialogFooter>
          </form>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle>API Key Created</DialogTitle>
              <DialogDescription>
                Please copy or download your API keys now. You won&apos;t be
                able to see them again!
              </DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="border rounded-lg p-4">
                <div className="flex items-center gap-2 mb-4">
                  <h3 className="font-medium">Platform API Key</h3>
                  <TooltipProvider delayDuration={0}>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <InfoIcon className="h-4 w-4 text-muted-foreground cursor-help" />
                      </TooltipTrigger>
                      <TooltipContent side="top">
                        <p>Use this key to authenticate with our platform</p>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                </div>
                <div className="flex flex-col gap-2">
                  <Label htmlFor="combined-key">Platform API Key</Label>
                  <div className="flex items-center gap-2">
                    <Input
                      id="combined-key"
                      readOnly
                      value={`nervemq_${apiKey?.access_key}_${apiKey?.secret_key}`}
                      type="text"
                      className="font-mono"
                    />
                    <TooltipProvider delayDuration={0}>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() =>
                              navigator.clipboard.writeText(apiKey?.token || "")
                            }
                          >
                            <CopyIcon className="h-4 w-4" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>
                          <p>Copy to clipboard</p>
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                  </div>
                </div>
              </div>

              <div className="border rounded-lg p-4">
                <div className="flex items-center gap-2 mb-4">
                  <h3 className="font-medium">AWS API Keys</h3>
                  <TooltipProvider delayDuration={0}>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <InfoIcon className="h-4 w-4 text-muted-foreground cursor-help" />
                      </TooltipTrigger>
                      <TooltipContent side="top">
                        <p>AWS SQS compatible credentials for queue access</p>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                </div>
                <div className="flex flex-col gap-4">
                  <div className="flex flex-col gap-2">
                    <Label htmlFor="access-key">Access Key</Label>
                    <div className="flex items-center gap-2">
                      <Input
                        id="access-key"
                        readOnly
                        value={apiKey?.access_key}
                        type="text"
                        className="font-mono"
                      />
                      <TooltipProvider delayDuration={0}>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() =>
                                navigator.clipboard.writeText(
                                  apiKey?.access_key || "",
                                )
                              }
                            >
                              <CopyIcon className="h-4 w-4" />
                            </Button>
                          </TooltipTrigger>
                          <TooltipContent>
                            <p>Copy to clipboard</p>
                          </TooltipContent>
                        </Tooltip>
                      </TooltipProvider>
                    </div>
                  </div>

                  <div className="flex flex-col gap-2">
                    <Label htmlFor="secret-key">Secret Key</Label>
                    <div className="flex items-center gap-2">
                      <Input
                        id="secret-key"
                        readOnly
                        value={apiKey?.secret_key}
                        type="text"
                        className="font-mono"
                      />
                      <TooltipProvider delayDuration={0}>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() =>
                                navigator.clipboard.writeText(
                                  apiKey?.secret_key || "",
                                )
                              }
                            >
                              <CopyIcon className="h-4 w-4" />
                            </Button>
                          </TooltipTrigger>
                          <TooltipContent>
                            <p>Copy to clipboard</p>
                          </TooltipContent>
                        </Tooltip>
                      </TooltipProvider>
                    </div>
                  </div>
                </div>
              </div>

              <div className="grid gap-2">
                <Button onClick={downloadKey}>Download Keys</Button>
              </div>
            </div>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}

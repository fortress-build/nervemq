import { type InferType, object, number, string } from "yup";

export const queueSettingsSchema = object({
  namespace: string().required(),
  queue: string().required(),
  maxRetries: number().required().min(0).max(10),
  timeout: number().required().min(1).max(300),
});

export type QueueSettingsType = InferType<typeof queueSettingsSchema>;
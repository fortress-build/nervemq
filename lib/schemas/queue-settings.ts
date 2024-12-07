import { type InferType, object, number, string } from "yup";

export const updateQueueConfigSchema = object({
  maxRetries: number().required().min(0).max(999),
  deadLetterQueue: string().optional(),
});

export type QueueConfig = InferType<typeof updateQueueConfigSchema>;

export type UpdateQueueConfigRequest = {
  queue: string;
  namespace: string;
  maxRetries: number;
  deadLetterQueue?: string;
};


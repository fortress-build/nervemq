import { z } from "zod";

export const deleteQueueSchema = z.object({
  name: z
    .string()
    .min(1)
    .max(32)
    .regex(/^[a-zA-Z0-9]+$/),
  namespace: z
    .string()
    .min(1)
    .max(32)
    .regex(/^[a-zA-Z0-9]+$/),
});

export type DeleteQueueRequest = z.infer<typeof deleteQueueSchema>;

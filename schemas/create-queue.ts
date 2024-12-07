import { z } from "zod";

export const createQueueSchema = z.object({
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
  attributes: z.map(z.string().min(1), z.string()),
  tags: z.map(z.string().min(1), z.string()),
});

export type CreateQueueRequest = z.infer<typeof createQueueSchema>;

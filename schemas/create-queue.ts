import { isAlphaNumeric } from "@/lib/utils";
import { type InferType, object, string } from "yup";

export const createQueueSchema = object({
  name: string()
    .required()
    .max(32)
    .min(1)
    .test("name", "name should be alphanumeric", isAlphaNumeric),
  namespace: string()
    .required()
    .max(32)
    .min(1)
    .test("namespace", "namespace should be alphanumeric", isAlphaNumeric),
});

export type CreateQueueRequest = InferType<typeof createQueueSchema>;

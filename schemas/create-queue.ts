import { isAlphaNumeric } from "@/lib/utils";
import { array, type InferType, object, string, tuple } from "yup";

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
  attributes: array()
    .of(tuple([string(), string()]).required())
    .default([]),
  tags: array()
    .of(tuple([string().required(), string().required()]).required())
    .default([]),
});

export type CreateQueueRequest = InferType<typeof createQueueSchema>;

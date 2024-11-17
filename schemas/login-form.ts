import { type InferType, object, string } from "yup";

export const loginFormShema = object({
  email: string().email().required(),
  password: string().min(8).max(32).required(),
});

export type LoginRequest = InferType<typeof loginFormShema>;

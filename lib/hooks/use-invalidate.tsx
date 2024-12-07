import { useQueryClient } from "@tanstack/react-query";

export function useInvalidate(queryKey: string[]) {
  const queryClient = useQueryClient();
  return () => {
    queryClient.invalidateQueries({ queryKey });
  };
}

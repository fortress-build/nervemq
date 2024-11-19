import { useEffect, useRef } from 'react';
import { useRouter } from 'next/navigation';
import { useGlobalState } from "@/lib/state/global";
import { SERVER_ENDPOINT } from "@/app/globals";

export function useVerifyUser(intervalMs: number = 300 * 1000) {
  const router = useRouter();
  const intervalRef = useRef<NodeJS.Timeout>();

  useEffect(() => {
    const verify = async () => {
      console.log('try')
      try {
        const response = await fetch(`${SERVER_ENDPOINT}/auth/verify`, {
            method: "POST",
            credentials: "include",
            mode: "cors",
        });

        if (!response.ok) {
          useGlobalState.setState({ session: undefined });
          router.push('/login');
          return;
        }

        const data = await response.json();
        useGlobalState.setState({ session: data });
      } catch (error) {
        useGlobalState.setState({ session: undefined });
        router.push('/login');
      }
    };

    verify(); // Run immediately
    intervalRef.current = setInterval(verify, intervalMs);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [intervalMs, router]);
}
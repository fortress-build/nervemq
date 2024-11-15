'use client'

import { signIn } from 'next-auth/react'
import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardHeader, CardContent } from "@/components/ui/card"
import { Label } from "@/components/ui/label"

export default function LoginPage() {
  const router = useRouter()
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const formData = new FormData(e.currentTarget)

    try {
      const response = await signIn('credentials', {
        username: formData.get('username'),
        password: formData.get('password'),
        redirect: false,
      })

      if (response?.error) {
        setError('Invalid credentials')
      } else {
        router.push('/queues')
        router.refresh()
      }
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (error) {
      setError('An error occurred during login')
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center">
      <Card className="w-96">
        <CardHeader>
          <h1 className="text-2xl font-bold">Login</h1>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit}>
            {error && (
              <div className="mb-4 text-red-500 text-sm">{error}</div>
            )}
            <div className="mb-4">
              <Label htmlFor="username">
                Username
              </Label>
              <Input
                type="text"
                id="username"
                name="username"
                className="w-full p-2 border rounded"
                required
              />
            </div>
            <div className="mb-6">
              <Label htmlFor="password">
                Password
              </Label>
              <Input
                type="password"
                id="password"
                name="password"
                className="w-full p-2 border rounded"
                required
              />
            </div>
            <Button
              type="submit"
              className="w-full"
            >
              Sign In
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}

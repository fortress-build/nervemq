import { type NextRequest, NextResponse } from "next/server";

export function middleware(request: NextRequest) {
  if (
    request.nextUrl.pathname.startsWith("/login") ||
    request.cookies.get("nervemq_session") !== undefined
  ) {
    if (request.nextUrl.pathname.startsWith("/queues")) {
      const split = request.nextUrl.pathname
        .split("/")
        .filter((s) => s.length > 0);
      if (split.length !== 1 && split.length !== 3) {
        return NextResponse.redirect(new URL("/queues", request.url));
      }
    }

    return NextResponse.next();
  }

  return NextResponse.redirect(new URL("/login", request.url));
}

export const config = {
  matcher: [
    "/((?!login|_next/static|_next/image|favicon.ico|sitemap.xml|robots.txt).*)",
  ],
};

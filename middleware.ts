import { type NextRequest, NextResponse } from "next/server";

export function middleware(request: NextRequest) {
  if (
    request.nextUrl.pathname.startsWith("/login") ||
    request.cookies.get("nervemq_session") !== undefined
  ) {
    return NextResponse.next();
  }

  return NextResponse.redirect(new URL("/login", request.url));
}

export const config = {
  matcher: [
    "/((?!login|_next/static|_next/image|favicon.ico|sitemap.xml|robots.txt).*)",
  ],
};

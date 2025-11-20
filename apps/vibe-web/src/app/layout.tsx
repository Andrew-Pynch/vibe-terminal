import type { ReactNode } from "react";

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body>
        <main style={{ padding: 16, fontFamily: "system-ui, sans-serif" }}>
          <h1>Vibe Project Sessions</h1>
          {children}
        </main>
      </body>
    </html>
  );
}

import type { Metadata } from 'next'
import { Inter } from 'next/font/google'
import './globals.css'
import { Providers } from './providers'

const inter = Inter({ subsets: ['latin'] })

export const metadata: Metadata = {
  title: 'Ruflo - AI Meeting Platform',
  description: 'Intelligent meeting transcription and insights',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <div className="bg-indigo-600 text-white text-center text-sm py-2 px-4">
          <span className="font-medium">Live Demo</span> — Mock data mode. 
          <a href="https://github.com/dexinox-jash/truffle" className="underline hover:text-indigo-200 ml-1" target="_blank" rel="noopener noreferrer">
            View source on GitHub →
          </a>
        </div>
        <Providers>{children}</Providers>
      </body>
    </html>
  )
}

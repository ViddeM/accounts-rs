import '../global-styles/globals.scss'
import '../global-styles/common.scss'
import '../global-styles/form.scss'
import type { AppProps } from 'next/app'

function MyApp({ Component, pageProps }: AppProps) {
  return <Component {...pageProps} />
}

export default MyApp

import "../resources/styles/maven_pro.scss";
import "../resources/styles/globals.scss";
import "../resources/styles/arvo.scss";
import "../resources/styles/vars.scss";

import type { AppProps } from "next/app";
import Head from "next/head";
import Header from "../components/views/header/Header";

function MyApp({ Component, pageProps }: AppProps) {
  return (
    <div>
      <Head>
        <title>Accounts-rs</title>
      </Head>

      <Header />
      <div>
        <Component {...pageProps} />
      </div>
    </div>
  );
}

export default MyApp;

import DefaultLayout from "../layouts/DefaultLayout";
import { useRouter } from "next/router";

const Home = () => {
  /* TODO: Add translations */
  const router = useRouter();

  if (typeof window !== "undefined") {
    router.push("/me").finally(() => {});
  }

  return <DefaultLayout>Hello and welcome to Accounts-RS</DefaultLayout>;
};

export default Home;

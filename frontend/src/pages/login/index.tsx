import CardLayout from "../../layouts/CardLayout";
import React from "react";
import { LOGIN_ENDPOINT } from "../../api/Endpoints";
import Link from "next/link";
import styles from "./index.module.scss";
import { useMe } from "../../hooks/useMe";
import { useRouter } from "next/router";

const Login = () => {
  const { me } = useMe();
  const router = useRouter();

  if (typeof window !== "undefined" && me) {
    router.push("/me");
  }

  return (
    <CardLayout>
      <div className={`card mainCard`}>
        <h2>User is unauthorized</h2>

        <Link href={LOGIN_ENDPOINT} className={styles.loginLink}>
          Login
        </Link>
      </div>
    </CardLayout>
  );
};

export default Login;

import React from "react";
import { useMe } from "../../hooks/useMe";
import CardLayout from "../../layouts/CardLayout";
import styles from "./index.module.scss";
import Link from "next/link";

const Me = () => {
  let { me } = useMe();

  return (
    <CardLayout>
      <div className="card">
        <h3>
          {/*TODO: fix translations*/}
          My account
        </h3>
        Name: {me?.firstName} {me?.lastName}
        <br />
        Email: {me?.email}
        <br />
        <Link href={"/api/forgot_password"}>
          <a className={styles.resetPasswordLink}>Reset password</a>
        </Link>
      </div>
    </CardLayout>
  );
};

export default Me;

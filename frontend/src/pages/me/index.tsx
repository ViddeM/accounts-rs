import React from "react";
import {useMe} from "../../hooks/useMe";
import CardLayout from "../../layouts/CardLayout";
import styles from "./index.module.scss";
import Link from "next/link";

const Me = () => {
    let {me} = useMe();

    return (
        <CardLayout>
            <div className={`card mainCard`}>
                <h2>
                    {/*TODO: fix translations*/}
                    My account
                </h2>

                <div className={styles.myInfo}>
                    <div className={styles.infoRow}>
                        <p>Name:</p> {me?.firstName} {me?.lastName}
                    </div>
                    <div className={styles.infoRow}>
                        <p>Email:</p> {me?.email}
                    </div>
                </div>
                <Link href={"/api/forgot_password"} className={styles.resetPasswordLink}>
                    Reset password
                </Link>
            </div>
        </CardLayout>
    );
};

export default Me;

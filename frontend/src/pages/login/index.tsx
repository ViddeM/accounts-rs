import CardLayout from "../../layouts/CardLayout";
import React from "react";
import {LOGIN_ENDPOINT} from "../../api/Endpoints";
import Link from "next/link";
import styles from "./index.module.scss";

const Login = () => {
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

import CardLayout from "../../layouts/CardLayout";
import styles from "./index.module.scss";
import {GetServerSideProps} from "next";
import {Api} from "../../api/Api";
import {OauthClient} from "../../api/OauthClient";
import React from "react";
import Link from "next/link";
import {CREATE_CLIENT_ENDPOINT} from "../../api/Endpoints";

type ClientsProps = {
    error?: boolean;
    clients?: OauthClient[];
}

const Clients = ({error, clients}: ClientsProps) => {
    if (error) {
        return <div>ERRROR ERROR</div>
    }

    if (!clients) {
        return <div>Loading...</div>
    }

    return (
        <CardLayout>
            <div className={`card mainCard`}>
                <h2>
                    Oauth clients
                </h2>

                {clients.map(client => (
                    <div className={styles.oauthClient} key={client.clientId}>
                        <b>
                            {client.clientName}
                        </b>
                        <div>
                            <b>
                                Client ID
                            </b>
                            <p>
                                {client.clientId}
                            </p>
                        </div>
                        <div>
                            <b>
                                Redirect URI
                            </b>
                            <p>
                                {client.redirectUri}
                            </p>
                        </div>
                    </div>
                ))}

                <Link href={CREATE_CLIENT_ENDPOINT} passHref={true}>
                    <a className={styles.createClientLink}>
                        Create client
                    </a>
                </Link>

            </div>
        </CardLayout>
    )
}

export const getServerSideProps: GetServerSideProps = async (ctx) => {
    let response = await Api.oauthClients.getAll(
        ctx?.req?.headers?.cookie ?? undefined
    );

    if (response.isError) {
        console.error("Failed to get oauth clients", response.error);
        return {
            props: {
                error: true,
            },
        };
    }

    return {
        props: {
            clients: response.data?.oauthClients,
        },
    };
};


export default Clients;
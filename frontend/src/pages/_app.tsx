import "../resources/styles/maven_pro.scss";
import "../resources/styles/globals.scss";
import "../resources/styles/arvo.scss";
import "../resources/styles/vars.scss";

import type {AppContext, AppProps} from "next/app";
import App from "next/app";
import Head from "next/head";
import Header from "../components/views/header/Header";
import {Api, isClientSide} from "../api/Api";
import React, {useState} from "react";
import {AuthContext} from "../hooks/useMe";
import {Me} from "../api/Me";
import ErrorHeader from "../components/views/error_header/ErrorHeader";
import {useRouter} from "next/router";
import {LOGIN_PAGE_ENDPOINT} from "../api/Endpoints";
import Modal, {ModalProps, trapTabKey} from "../components/views/modal/Modal";
import {ModalContext} from "../hooks/useModal";

type MyAppProps = AppProps & {
    me: Me;
    error?: boolean;
    failedToReachBackend: boolean;
};

function MyApp({Component, pageProps, me, error, failedToReachBackend}: MyAppProps) {
    const router = useRouter();
    if (error && router.pathname !== LOGIN_PAGE_ENDPOINT && isClientSide()) {
        router.push(LOGIN_PAGE_ENDPOINT).then(() => {
            router.reload();
        });
    }

    const [modalProps, setModalProps] = useState<ModalProps | undefined>(
        undefined
    );

    return (
        <AuthContext.Provider value={{me: me}}>
            <ModalContext.Provider
                value={{
                    openModal: (props) =>
                        setModalProps(handleOpenModalProps(props, setModalProps)),
                }}
            >

                <Head>
                    {/* TODO: Add translations */}
                    <title>Accounts-rs</title>
                </Head>

                {modalProps && <Modal {...modalProps} />}

                <div
                    aria-hidden={modalProps ? "true" : undefined}
                    className="fullHeight"
                >
                    {failedToReachBackend && <ErrorHeader/>}
                    <Header/>
                    <div>
                        <Component {...pageProps} />
                    </div>
                </div>
            </ModalContext.Provider>
        </AuthContext.Provider>
    );
}

// @ts-ignore
MyApp.getInitialProps = async (appContext: AppContext) => {
    const ret = await App.getInitialProps(appContext);
    const {ctx} = appContext;
    const {res} = ctx;

    let response = await Api.me.getMe(ctx?.req?.headers?.cookie ?? undefined);

    if (response.isError) {
        return {
            ...ret,
            error: true,
            failedToReachBackend: response.failedToReachBackend,
        };
    }

    if (response.redirect) {
        const url = response.redirect;
        if (res) {
            res.writeHead(302, {Location: url});
            res.end();
        } else {
            window.location.href = url;
        }
    }

    return {
        ...ret,
        me: response.data,
    };
};

// TODO: Move this to Modal.tsx
function handleOpenModalProps(
    props: ModalProps,
    setModalProps: (modalProps: ModalProps | undefined) => void
): ModalProps {
    document.body.style.overflow = "hidden";

    const closeOnEsc = ({key}: KeyboardEvent) => {
        if (key === "Escape") {
            closeModal(props.onClose);
        }
    };

    const closeModal = (onClose: (() => void) | undefined) => {
        document.body.style.overflow = "auto";
        setModalProps(undefined);
        window.removeEventListener("keydown", closeOnEsc);
        window.removeEventListener("keydown", trapTabKey);

        if (onClose) {
            onClose();
        }
    };

    window.addEventListener("keydown", closeOnEsc);
    window.addEventListener("keydown", trapTabKey);

    return {
        ...props,
        onClose: () => {
            closeModal(props.onClose);
        },
        declineButton: props.declineButton
            ? {
                ...props.declineButton,
                onClick: () => {
                    closeModal(props.declineButton?.onClick);
                },
            }
            : undefined,
        confirmButton: props.confirmButton
            ? {
                ...props.confirmButton,
                onClick: () => {
                    closeModal(props.confirmButton?.onClick);
                },
            }
            : undefined,
    };
}

export default MyApp;

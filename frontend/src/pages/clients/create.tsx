import CardLayout from "../../layouts/CardLayout";
import TextField from "../../components/elements/TextField/TextField";
import styles from "./create.module.scss";
import { Button } from "../../components/elements/Buttons/Buttons";
import { Api } from "../../api/Api";
import { FormEvent, useState } from "react";
import { useRouter } from "next/router";
import { useModal } from "../../hooks/useModal";
import { CLIENTS_ENDPOINT } from "../../api/Endpoints";

const CLIENT_NAME_ID = "CLIENT_NAME";
const REDIRECT_URI_ID = "REDIRECT_URI_ID";

const CreateClient = () => {
  const router = useRouter();
  const [clientName, setClientName] = useState("");
  const [redirectUri, setRedirectUri] = useState("");
  const [emailScope, setEmailScope] = useState(false);
  const [openidScope, setOpenidScope] = useState(false);
  const [error, setError] = useState<undefined | string>(undefined);
  const { openModal } = useModal();

  const scopes = [emailScope ? "email" : null, openidScope ? "openid" : null]
    .filter((s) => s !== null)
    .map((s) => s!!);

  const onSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    Api.oauthClients
      .create(clientName, redirectUri, scopes)
      .then((resp) => {
        openModal({
          title: "New Client info",
          content: `The client has been created successfully, below you will find the client id and client secret necessary later.
Make sure to write these down as the client secret will not be viewable after this point.
                
Client ID: ${resp.data!!.clientId}
                
Client Secret: ${resp.data!!.clientSecret}`,
          confirmButton: {
            text: "Ok",
            onClick: () => {
              router.push(CLIENTS_ENDPOINT).then(() => {});
            },
          },
          onClose: () => {
            router.push(CLIENTS_ENDPOINT).then(() => {});
          },
        });
      })
      .catch((err) => {
        setError(err.error);
      });
  };

  return (
    <CardLayout>
      <form className={`card mainCard`} onSubmit={onSubmit}>
        <h2>Create oauth client</h2>

        <div className={styles.createClientRow}>
          <label htmlFor={CLIENT_NAME_ID}>Client Name:</label>
          <TextField
            id={CLIENT_NAME_ID}
            name={CLIENT_NAME_ID}
            type="text"
            value={clientName}
            onChange={(e) => {
              setClientName(e.target.value);
            }}
          />
        </div>
        <div className={styles.createClientRow}>
          <label htmlFor={REDIRECT_URI_ID}>Redirect Uri:</label>
          <TextField
            id={REDIRECT_URI_ID}
            name={REDIRECT_URI_ID}
            value={redirectUri}
            onChange={(e) => {
              setRedirectUri(e.target.value);
            }}
          />
        </div>

        <fieldset className={styles.scopesContainer}>
          <legend>
            <h4>Scopes</h4>
          </legend>
          <ul className={styles.scopesList}>
            <li>
              <span>
                <input
                  id="email_scope"
                  type="checkbox"
                  checked={emailScope}
                  onChange={() => {
                    setEmailScope(!emailScope);
                  }}
                />
                <label htmlFor="email_scope">Email</label>
              </span>
            </li>
            <li>
              <span>
                <input
                  id="openid_scope"
                  type="checkbox"
                  checked={openidScope}
                  onChange={() => setOpenidScope(!openidScope)}
                />
                <label htmlFor="openid_scope">OpenID</label>
              </span>
            </li>
          </ul>
        </fieldset>

        {<p style={{ backgroundColor: "red" }}>{error}</p>}

        <Button
          size="normal"
          variant="primary"
          className="marginTop"
          type="submit"
        >
          Create
        </Button>
      </form>
    </CardLayout>
  );
};

export default CreateClient;

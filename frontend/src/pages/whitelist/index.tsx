import CardLayout from "../../layouts/CardLayout";
import { GetServerSideProps } from "next";
import { Api } from "../../api/Api";
import { IconButton } from "../../components/elements/Buttons/Buttons";
import { faTrashCan } from "@fortawesome/free-regular-svg-icons";
import TextField from "../../components/elements/TextField/TextField";
import { faAdd } from "@fortawesome/free-solid-svg-icons";
import styles from "./index.module.scss";
import { useState } from "react";
import { useRouter } from "next/router";

type WhitelistProps = {
  error?: boolean;
  whitelist?: string[];
};

const Whitelist = ({ error, whitelist }: WhitelistProps) => {
  const router = useRouter();
  const [email, setEmail] = useState("");

  if (error) {
    return <div>ERRRRROR</div>;
  }

  if (!whitelist) {
    return <div>Loading...</div>;
  }

  const onSubmit = (e) => {
    e.preventDefault();
    Api.whitelist
      .addToWhitelist(email)
      .then((_) => {
        router.replace(router.asPath).then((_) => {});
      })
      .catch((err) => {
        console.log("FUCK, err: ", err);
      });
  };

  return (
    <CardLayout>
      <div className={`card mainCard ${styles.addToWhitelistCard}`}>
        <h2>Whitelist</h2>
        <form onSubmit={onSubmit}>
          <table>
            <thead>
              <tr>
                <th align="left">Email</th>
                <th align="center">Delete?</th>
              </tr>
            </thead>
            <tbody>
              {whitelist.map((e) => (
                <tr key={e}>
                  <td align="left">{e}</td>
                  <td align="center">
                    <IconButton
                      icon={faTrashCan}
                      size="small"
                      variant="opaque"
                      type="button"
                      onClick={() => {
                        let c = confirm(
                          "Are you sure you want to remove this email from the whitelist?"
                        );
                        if (c) {
                          Api.whitelist
                            .removeFromWhitelist(e)
                            .then((_) => {
                              router.replace(router.asPath).then((_) => {});
                            })
                            .catch((err) => {
                              console.log("Feckery and buggery, err: ", err);
                            });
                        }
                      }}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
            <tfoot>
              <td className={styles.addToWhitelistRow}>
                <label>Email:</label>
                <TextField
                  placeholder={"Email to whitelist"}
                  maxLength={100}
                  autoComplete="email"
                  spellCheck={false}
                  type="email"
                  value={email}
                  onChange={(e) => {
                    setEmail(e.target.value);
                  }}
                />
              </td>
              <td align="center">
                <IconButton
                  type="submit"
                  icon={faAdd}
                  size="small"
                  variant="opaque"
                />
              </td>
            </tfoot>
          </table>
        </form>
      </div>
    </CardLayout>
  );
};

export const getServerSideProps: GetServerSideProps = async (ctx) => {
  let response = await Api.whitelist.getAll(
    ctx?.req?.headers?.cookie ?? undefined
  );
  if (response.isError) {
    console.error("Failed to get whitelist", response.error);
    return {
      props: {
        error: true,
      },
    };
  }

  return {
    props: {
      whitelist: response.data?.emails,
    },
  };
};

export default Whitelist;

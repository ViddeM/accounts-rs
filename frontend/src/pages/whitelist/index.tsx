import CardLayout from "../../layouts/CardLayout";
import { GetServerSideProps } from "next";
import { Api } from "../../api/Api";
import { IconButton } from "../../components/elements/Buttons/Buttons";
import { faTrashCan } from "@fortawesome/free-regular-svg-icons";
import TextField from "../../components/elements/TextField/TextField";
import { faAdd } from "@fortawesome/free-solid-svg-icons";
import styles from "./index.module.scss";

type WhitelistProps = {
  error?: boolean;
  whitelist?: string[];
};

const Whitelist = ({ error, whitelist }: WhitelistProps) => {
  if (error) {
    return <div>ERRRRROR</div>;
  }

  if (!whitelist) {
    return <div>Loading...</div>;
  }

  return (
    <CardLayout>
      <div className={`card`}>
        <h4>Whitelist</h4>
        <form>
          <table>
            <thead>
              <tr>
                <th>Email</th>
                <th>Delete?</th>
              </tr>
            </thead>
            <tbody>
              {whitelist.map((e) => (
                <tr key={e}>
                  <td>{e}</td>
                  <td>
                    <IconButton
                      icon={faTrashCan}
                      size="small"
                      variant="opaque"
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
                />
              </td>
              <td>
                <IconButton icon={faAdd} size="small" variant="opaque" />
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

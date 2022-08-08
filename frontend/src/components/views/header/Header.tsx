import styles from "./Header.module.scss";
import Link from "next/link";
import { useRouter } from "next/router";
import {
  CLIENTS_ENDPOINT,
  ME_ENDPOINT,
  USERS_ENDPOINT,
  WHITELIST_ENDPOINT,
} from "../../../api/Endpoints";
import { Button } from "../../elements/Buttons/Buttons";
import { Api } from "../../../api/Api";
import { useMe } from "../../../hooks/useMe";
import { AuthorityLevel } from "../../../api/AuthorityLevel";

const Header = () => {
  const { pathname, push } = useRouter();
  const { me } = useMe();

  return (
    <header className={styles.headerContainer}>
      <div className={styles.header}>
        <Link href={"/"} passHref={true}>
          <a>
            {/* TODO: Add translations */}
            <h1 className={styles.headerTitle}>Accounts-RS</h1>
          </a>
        </Link>
        <Button
          size={"normal"}
          variant={"outlined"}
          onClick={() => {
            Api.me
              .postLogout()
              .then(() => push("/"))
              .catch((err) => {
                console.error("Error during logout", err);
                //  TODO: Handle error
              });
          }}
        >
          Logout
        </Button>
      </div>
      <ul className={styles.subHeader}>
        <li
          className={
            pathname === ME_ENDPOINT ? styles.selectedSubHeaderItem : ""
          }
        >
          <Link href={"/me"}>
            {/* TODO: Add translations */}
            <a>My account</a>
          </Link>
        </li>
        {me?.authority === AuthorityLevel.Admin && (
          <>
            <li
              className={
                pathname === USERS_ENDPOINT ? styles.selectedSubHeaderItem : ""
              }
            >
              <Link href={"/users"}>
                {/* TODO: Add translations */}
                <a>Users</a>
              </Link>
            </li>
            <li
              className={
                pathname === CLIENTS_ENDPOINT
                  ? styles.selectedSubHeaderItem
                  : ""
              }
            >
              <Link href={"/clients"}>
                {/* TODO: Add translations */}
                <a>Clients</a>
              </Link>
            </li>
            <li
              className={
                pathname === WHITELIST_ENDPOINT
                  ? styles.selectedSubHeaderItem
                  : ""
              }
            >
              <Link href={"/whitelist"}>
                {/* TODO: Add translations */}
                <a>Whitelist</a>
              </Link>
            </li>
          </>
        )}
      </ul>
    </header>
  );
};

export default Header;

import styles from "./Header.module.scss";

const Header = () => {
  return (
    <div className={styles.headerContainer}>
      <h1 className={styles.headerTitle}>Accounts-RS</h1>
    </div>
  );
};

export default Header;

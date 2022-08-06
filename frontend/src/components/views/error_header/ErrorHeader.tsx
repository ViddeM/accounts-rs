import styles from "./ErrorHeader.module.scss";

const ErrorHeader = () => {
  return (
    <div className={styles.errorHeaderContainer}>
      {/* TODO: Add translations */}
      <h3>Failed to communicate with the server</h3>
    </div>
  );
};

export default ErrorHeader;

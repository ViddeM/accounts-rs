import CardLayout from "../../layouts/CardLayout";
import { GetServerSideProps } from "next";
import { Api } from "../../api/Api";
import { User } from "../../api/User";
import styles from "./index.module.scss";

type UsersProps = {
  error?: boolean;
  users?: User[];
};

const Users = ({ users, error }: UsersProps) => {
  if (error) {
    return <div>FUck</div>;
  }

  if (!users) {
    return <div>Loading?</div>;
  }

  return (
    <CardLayout>
      <div className={`card ${styles.users}`}>
        <h4>Users</h4>
        <p>
          Displaying {users.length} user{users.length > 1 ? "s" : ""}
        </p>
        <table className={styles.usersTable}>
          <thead>
            <tr>
              <th align="left">First Name</th>
              <th align="left">Last Name</th>
              <th align="right">Authority</th>
            </tr>
          </thead>
          <tbody>
            {users.map((user, index) => (
              <tr key={index}>
                <td>{user.firstName}</td>
                <td>{user.lastName}</td>
                <td align="right">{user.authority}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </CardLayout>
  );
};

export const getServerSideProps: GetServerSideProps = async (ctx) => {
  let response = await Api.user.getAll(ctx?.req?.headers?.cookie ?? undefined);
  if (response.isError) {
    console.error("Failed to get all users", response.error);
    return {
      props: {
        error: true,
      },
    };
  }

  return {
    props: {
      users: response.data?.users,
    },
  };
};

export default Users;

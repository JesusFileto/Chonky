import { FormEvent, useState, VoidFunctionComponent } from "react";
import { useCreatePage } from "../../hooks/useCreatePage";
import { Modal } from "../Modal";

import styles from "./CreatePage.module.scss";
import { Button } from "../../forms/Button";

type CreatePageProps = {
  close: () => void;
  accountId: string;
};

export const CreatePage: VoidFunctionComponent<CreatePageProps> = ({
  close,
  accountId,
}) => {
  const [title, setTitle] = useState("");
  const [loading, setLoading] = useState(false);

  const { create } = useCreatePage();

  const createPage = (event: FormEvent) => {
    event.preventDefault();
    setLoading(true);
    create({
      variables: { accountId, properties: { title } },
    })
      .catch((err) => console.error("Could not create page: ", err))
      .finally(() => {
        setLoading(false);
        close();
      });
  };

  return (
    <Modal show close={close}>
      <form className={styles.CreatePage} onSubmit={createPage}>
        <h2>Don't be afraid of a blank page...</h2>

        {/* eslint-disable-next-line jsx-a11y/label-has-associated-control */}
        <label>Title</label>
        <input
          onChange={(event) => setTitle(event.target.value)}
          placeholder="What is this document?"
          required
          type="text"
          value={title}
        />

        <Button disabled={loading} big type="submit">
          Create
        </Button>
      </form>
    </Modal>
  );
};
